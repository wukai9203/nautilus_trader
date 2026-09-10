//! Live market data client.
//!
//! # Only bars, and only closed ones
//!
//! The venue's stream carries a candle channel; this client publishes from it and serves
//! historical bars over REST. Quotes, trades and order books are not implemented, so the
//! trait's defaults leave them logged and unhandled rather than silently pretending.
//!
//! Every push on the candle channel republishes the bar that is currently forming, so most
//! frames describe a bar whose high, low and close can still move. Only completed bars are
//! published. A strategy that acted on a forming bar would be deciding from values that were
//! not knowable at that timestamp, and its live results would not be comparable with any
//! backtest — the same look-ahead the historical path removes with `drop_forming_tail`.
//!
//! Completion cannot be read off the venue's `closed` flag alone. Observed on the live
//! testnet across two full bar periods: the flag was never set, and the bar simply rolled —
//! each push carried an open time one interval later than the last. A client that waited for
//! the flag would publish nothing at all while its connection looked healthy, which is the
//! worst failure available here, because it is indistinguishable from a market with no
//! trades. So the last push of each bar is held, and released when either the flag is set or
//! a push for a later bar proves the held one's window has elapsed. The flag stays
//! authoritative when present; the successor is the fallback evidence.
//!
//! The cost is one bar of latency: a bar is published when the next one starts rather than
//! the instant it closes. That is inherent to the venue's behaviour, not a choice — there is
//! no earlier moment at which completion is observable.
//!
//! # Pushes are matched to the subscription that asked for them
//!
//! A candle names its instrument and interval as text. Rebuilding a bar type from those two
//! strings would publish data for anything the venue happened to send, including a channel
//! nobody subscribed to. Instead each subscription records the exact bar type the engine
//! asked for, and a push with no matching entry is dropped.

use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use async_trait::async_trait;
use nautilus_common::{
    clients::DataClient,
    live::{get_runtime, runner::get_data_event_sender},
    messages::{
        DataEvent,
        data::{
            BarsResponse, DataResponse, InstrumentResponse, InstrumentsResponse, RequestBars,
            RequestInstrument, RequestInstruments, SubscribeBars, UnsubscribeBars,
        },
    },
    providers::InstrumentProvider,
};
use nautilus_core::{
    UnixNanos,
    datetime::datetime_to_unix_nanos,
    time::{AtomicTime, get_atomic_clock_realtime},
};
use nautilus_model::{
    data::{BarType, Data},
    identifiers::{ClientId, Venue},
    instruments::{Instrument, InstrumentAny},
};
use parking_lot::Mutex;

use super::{
    history::{BarRequest, fetch_bars},
    parse::{parse_completed_bar, spec_to_interval},
};
use crate::{
    common::Market,
    config::SodexDataClientConfig,
    http::SodexHttpClient,
    providers::SodexInstrumentProvider,
    websocket::{Candle, CandleParams, SodexWebSocketClient, SodexWsEvent},
};

/// Identifies a candle feed the way the venue's push frames do.
type FeedKey = (String, String);

/// The most recent push for one feed, held until its bar is provably complete.
type Pending = HashMap<FeedKey, Candle>;

/// Live market data client for one SoDEX engine.
pub struct SodexDataClient {
    client_id: ClientId,
    venue: Venue,
    market: Market,
    config: SodexDataClientConfig,
    http: Arc<SodexHttpClient>,
    provider: SodexInstrumentProvider,
    ws: Option<Arc<SodexWebSocketClient>>,
    /// Bar types by the symbol and interval the venue will echo back on the stream.
    feeds: Arc<Mutex<HashMap<FeedKey, BarType>>>,
    is_connected: Arc<AtomicBool>,
    stream_task: Option<tokio::task::JoinHandle<()>>,
    data_sender: tokio::sync::mpsc::UnboundedSender<DataEvent>,
    clock: &'static AtomicTime,
}

impl std::fmt::Debug for SodexDataClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SodexDataClient")
            .field("client_id", &self.client_id)
            .field("venue", &self.venue)
            .field("connected", &self.is_connected.load(Ordering::Relaxed))
            .field("feeds", &self.feeds.lock().len())
            .finish()
    }
}

impl SodexDataClient {
    /// Creates a client for one engine on one network.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be built.
    pub fn new(client_id: ClientId, config: SodexDataClientConfig) -> anyhow::Result<Self> {
        let http = SodexHttpClient::new_public(config.network, config.market)
            .map_err(|e| anyhow::anyhow!("failed to build HTTP client: {e}"))?;
        let provider = SodexInstrumentProvider::new(config.network, config.market)?;

        Ok(Self {
            client_id,
            venue: config.venue(),
            market: config.market,
            http: Arc::new(http),
            provider,
            ws: None,
            feeds: Arc::new(Mutex::new(HashMap::new())),
            is_connected: Arc::new(AtomicBool::new(false)),
            stream_task: None,
            data_sender: get_data_event_sender(),
            clock: get_atomic_clock_realtime(),
            config,
        })
    }

    fn ws_client(&self) -> anyhow::Result<Arc<SodexWebSocketClient>> {
        self.ws
            .as_ref()
            .map(Arc::clone)
            .ok_or_else(|| anyhow::anyhow!("SoDEX data client is not connected"))
    }

    fn send(&self, event: DataEvent) {
        if let Err(e) = self.data_sender.send(event) {
            log::error!("sodex_data_event_undeliverable error={e}");
        }
    }
}

/// The venue symbol and interval a bar type maps to on the stream.
///
/// # Errors
///
/// Returns an error if this engine has no interval for the bar specification. Refusing here
/// keeps an unsupported request from becoming a subscription the venue silently never fills.
fn feed_key(bar_type: &BarType, market: Market) -> anyhow::Result<FeedKey> {
    let spec = bar_type.spec();
    let interval = spec_to_interval(&spec, market)
        .map_err(|e| anyhow::anyhow!("unsupported bar specification {spec}: {e}"))?;
    Ok((bar_type.instrument_id().symbol.to_string(), interval.to_string()))
}

/// Publishes completed bars from the stream and reports what the venue refused.
async fn run_stream(
    mut events: tokio::sync::mpsc::UnboundedReceiver<SodexWsEvent>,
    feeds: Arc<Mutex<HashMap<FeedKey, BarType>>>,
    sender: tokio::sync::mpsc::UnboundedSender<DataEvent>,
    clock: &'static AtomicTime,
) {
    let mut pending = Pending::new();

    while let Some(event) = events.recv().await {
        match event {
            SodexWsEvent::Candle(candle) => {
                let key = (candle.symbol.clone(), candle.interval.clone());
                let Some(bar_type) = feeds.lock().get(&key).copied() else {
                    log::debug!(
                        "sodex_candle_unsubscribed symbol={} interval={}",
                        candle.symbol,
                        candle.interval
                    );
                    continue;
                };

                for completed in advance(&mut pending, key, *candle) {
                    match parse_completed_bar(&completed, bar_type, clock.get_time_ns()) {
                        Ok(bar) => {
                            if sender.send(DataEvent::Data(Data::Bar(bar))).is_err() {
                                log::debug!("sodex_data_consumer_gone");
                                return;
                            }
                        }
                        Err(e) => {
                            log::error!("sodex_bar_unparsed bar_type={bar_type} error={e}");
                        }
                    }
                }
            }
            SodexWsEvent::RequestRejected { op, params, reason } => {
                // A refused subscribe leaves a feed permanently silent, which otherwise looks
                // exactly like a market with no trades.
                log::error!("sodex_stream_request_rejected op={op:?} params={params:?} {reason}");
            }
            SodexWsEvent::Reconnected => {
                // The held bars belong to the connection that just went away. A gap in the
                // feed means the held snapshot may no longer be that bar's final state, so
                // publishing it later would present a partial bar as a complete one.
                pending.clear();
                log::info!("sodex_stream_reconnected pending_bars_dropped");
            }
        }
    }
}

/// Folds one push into the held state, returning whatever became complete.
///
/// Completion has two sources, and this is where they are reconciled: the venue's own flag,
/// and the arrival of a push for a later bar. A push for an *earlier* bar than the one held
/// is discarded — the venue republishes the forming bar constantly, so an out-of-order frame
/// would otherwise rewrite a bar that has already been published.
fn advance(pending: &mut Pending, key: FeedKey, candle: Candle) -> Vec<Candle> {
    if candle.is_final() {
        // The flag is authoritative. Anything still held for this feed is at or before this
        // bar, so it is superseded rather than published twice.
        let held = pending.remove(&key);
        let mut completed = Vec::new();
        if let Some(held) = held
            && held.open_time_ms < candle.open_time_ms
        {
            completed.push(held);
        }
        completed.push(candle);
        return completed;
    }

    match pending.insert(key.clone(), candle) {
        Some(held) => {
            let candle = &pending[&key];
            if held.open_time_ms < candle.open_time_ms {
                // The held bar's window has elapsed: its successor is already forming.
                vec![held]
            } else if held.open_time_ms > candle.open_time_ms {
                log::debug!(
                    "sodex_candle_out_of_order held={} received={}",
                    held.open_time_ms,
                    candle.open_time_ms
                );
                pending.insert(key, held);
                Vec::new()
            } else {
                // A later snapshot of the same forming bar, which has just replaced it.
                Vec::new()
            }
        }
        None => Vec::new(),
    }
}

#[async_trait(?Send)]
impl DataClient for SodexDataClient {
    fn client_id(&self) -> ClientId {
        self.client_id
    }

    fn venue(&self) -> Option<Venue> {
        Some(self.venue)
    }

    fn start(&mut self) -> anyhow::Result<()> {
        log::info!(
            "sodex_data_client_start client_id={} venue={} network={:?}",
            self.client_id,
            self.venue,
            self.config.network
        );
        Ok(())
    }

    fn stop(&mut self) -> anyhow::Result<()> {
        log::info!("sodex_data_client_stop client_id={}", self.client_id);
        if let Some(task) = self.stream_task.take() {
            task.abort();
        }
        self.is_connected.store(false, Ordering::Relaxed);
        Ok(())
    }

    fn reset(&mut self) -> anyhow::Result<()> {
        self.stop()?;

        // Dropping the recorded feeds without closing the socket would leave the venue
        // pushing candles this client could no longer match to a bar type. The trait's reset
        // is synchronous, so the close is handed to the runtime rather than skipped.
        if let Some(ws) = self.ws.take() {
            get_runtime().spawn(async move { ws.close().await });
        }
        self.feeds.lock().clear();
        Ok(())
    }

    fn dispose(&mut self) -> anyhow::Result<()> {
        self.stop()
    }

    fn is_connected(&self) -> bool {
        self.is_connected.load(Ordering::Relaxed)
    }

    fn is_disconnected(&self) -> bool {
        !self.is_connected()
    }

    async fn connect(&mut self) -> anyhow::Result<()> {
        if self.is_connected() {
            return Ok(());
        }

        // Instruments first: a bar cannot be published for an instrument the engine has never
        // seen, and the symbol ids loaded here are what order submission later needs.
        self.provider.load_all(None).await?;

        let ws = Arc::new(SodexWebSocketClient::new(
            self.config.network,
            self.config.market,
        ));
        let events = ws.connect().await?;

        self.stream_task = Some(get_runtime().spawn(run_stream(
            events,
            Arc::clone(&self.feeds),
            self.data_sender.clone(),
            self.clock,
        )));
        self.ws = Some(ws);
        self.is_connected.store(true, Ordering::Relaxed);

        log::info!(
            "sodex_data_client_connected venue={} instruments={}",
            self.venue,
            self.provider.len()
        );
        Ok(())
    }

    async fn disconnect(&mut self) -> anyhow::Result<()> {
        if let Some(ws) = self.ws.take() {
            ws.close().await;
        }
        if let Some(task) = self.stream_task.take() {
            task.abort();
        }
        self.feeds.lock().clear();
        self.is_connected.store(false, Ordering::Relaxed);
        Ok(())
    }

    fn subscribe_bars(&mut self, cmd: SubscribeBars) -> anyhow::Result<()> {
        let bar_type = cmd.bar_type;
        let (symbol, interval) = feed_key(&bar_type, self.market)?;
        // Resolved before anything is recorded, so a call made while disconnected leaves no
        // entry claiming a feed that was never requested.
        let ws = self.ws_client()?;

        // Recorded before the request goes out: the venue can push the first candle before it
        // acknowledges the subscribe, and a push with no entry here would be dropped.
        self.feeds
            .lock()
            .insert((symbol.clone(), interval.clone()), bar_type);

        let params = CandleParams::new(symbol, interval);

        get_runtime().spawn(async move {
            if let Err(e) = ws.subscribe_candles(params).await {
                log::error!("sodex_subscribe_bars_failed bar_type={bar_type} error={e}");
            }
        });
        Ok(())
    }

    fn unsubscribe_bars(&mut self, cmd: &UnsubscribeBars) -> anyhow::Result<()> {
        let bar_type = cmd.bar_type;
        let (symbol, interval) = feed_key(&bar_type, self.market)?;
        let ws = self.ws_client()?;

        self.feeds
            .lock()
            .remove(&(symbol.clone(), interval.clone()));

        let params = CandleParams::new(symbol, interval);

        get_runtime().spawn(async move {
            if let Err(e) = ws.unsubscribe_candles(&params).await {
                log::error!("sodex_unsubscribe_bars_failed bar_type={bar_type} error={e}");
            }
        });
        Ok(())
    }

    fn request_bars(&self, request: RequestBars) -> anyhow::Result<()> {
        let http = Arc::clone(&self.http);
        let sender = self.data_sender.clone();
        let market = self.market;
        let clock = self.clock;
        let client_id = request.client_id.unwrap_or(self.client_id);
        let bar_type = request.bar_type;
        let start_nanos = datetime_to_unix_nanos(request.start);
        let end_nanos = datetime_to_unix_nanos(request.end);
        let params = request.params;
        let request_id = request.request_id;

        let bar_request = BarRequest {
            instrument_id: bar_type.instrument_id(),
            spec: bar_type.spec(),
            start_ms: start_nanos.map(unix_nanos_to_millis),
            end_ms: end_nanos.map(unix_nanos_to_millis),
            limit: request.limit.map(|n| u32::try_from(n.get()).unwrap_or(u32::MAX)),
        };

        get_runtime().spawn(async move {
            match fetch_bars(&http, market, &bar_request).await {
                Ok(bars) => {
                    let response = DataResponse::Bars(BarsResponse::new(
                        request_id,
                        client_id,
                        bar_type,
                        bars,
                        start_nanos,
                        end_nanos,
                        clock.get_time_ns(),
                        params,
                    ));
                    if let Err(e) = sender.send(DataEvent::Response(response)) {
                        log::error!("sodex_bars_response_undeliverable error={e}");
                    }
                }
                Err(e) => log::error!("sodex_bars_request_failed bar_type={bar_type} error={e}"),
            }
        });
        Ok(())
    }

    fn request_instruments(&self, request: RequestInstruments) -> anyhow::Result<()> {
        let instruments: Vec<InstrumentAny> =
            self.provider.store().get_all().values().cloned().collect();
        let response = DataResponse::Instruments(InstrumentsResponse::new(
            request.request_id,
            request.client_id.unwrap_or(self.client_id),
            self.venue,
            instruments,
            datetime_to_unix_nanos(request.start),
            datetime_to_unix_nanos(request.end),
            self.clock.get_time_ns(),
            request.params,
        ));
        self.send(DataEvent::Response(response));
        Ok(())
    }

    fn request_instrument(&self, request: RequestInstrument) -> anyhow::Result<()> {
        let Some(instrument) = self.provider.store().find(&request.instrument_id) else {
            log::warn!("sodex_instrument_unknown id={}", request.instrument_id);
            return Ok(());
        };

        let response = DataResponse::Instrument(Box::new(InstrumentResponse::new(
            request.request_id,
            request.client_id.unwrap_or(self.client_id),
            instrument.id(),
            instrument.clone(),
            datetime_to_unix_nanos(request.start),
            datetime_to_unix_nanos(request.end),
            self.clock.get_time_ns(),
            request.params,
        )));
        self.send(DataEvent::Response(response));
        Ok(())
    }
}

/// Converts a Nautilus timestamp into the milliseconds the venue's REST API expects.
const fn unix_nanos_to_millis(ts: UnixNanos) -> u64 {
    ts.as_u64() / 1_000_000
}

#[cfg(test)]
mod tests {
    use nautilus_model::{
        data::BarSpecification,
        enums::{AggregationSource, BarAggregation, PriceType},
        identifiers::{InstrumentId, Symbol},
    };

    use super::*;
    use crate::config::SODEX_PERPS;

    fn bar_type(step: usize, aggregation: BarAggregation) -> BarType {
        BarType::new(
            InstrumentId::new(Symbol::from("vBTC_vUSDC"), Venue::from(SODEX_PERPS)),
            BarSpecification::new(step, aggregation, PriceType::Last),
            AggregationSource::External,
        )
    }

    fn candle(closed: bool) -> Candle {
        candle_at(1_767_972_900_000, closed)
    }

    fn candle_at(open_time_ms: u64, closed: bool) -> Candle {
        Candle {
            open_time_ms,
            update_time_ms: 1_767_972_960_000,
            symbol: "vBTC_vUSDC".to_string(),
            interval: "1m".to_string(),
            open: "91869".to_string(),
            high: "91982".to_string(),
            low: "91869".to_string(),
            close: "91976".to_string(),
            volume: "4.12298".to_string(),
            quote_volume: "379148.6798".to_string(),
            trade_count: 12,
            closed,
        }
    }

    fn feeds_with(entry: BarType) -> Arc<Mutex<HashMap<FeedKey, BarType>>> {
        let mut feeds = HashMap::new();
        feeds.insert(("vBTC_vUSDC".to_string(), "1m".to_string()), entry);
        Arc::new(Mutex::new(feeds))
    }

    /// Drives the stream task over one batch of events and collects what it published.
    async fn publish(
        events: Vec<SodexWsEvent>,
        feeds: Arc<Mutex<HashMap<FeedKey, BarType>>>,
    ) -> Vec<DataEvent> {
        let (events_tx, events_rx) = tokio::sync::mpsc::unbounded_channel();
        for event in events {
            events_tx.send(event).unwrap();
        }
        drop(events_tx);

        let (data_tx, mut data_rx) = tokio::sync::mpsc::unbounded_channel();
        run_stream(events_rx, feeds, data_tx, get_atomic_clock_realtime()).await;

        let mut published = Vec::new();
        while let Ok(event) = data_rx.try_recv() {
            published.push(event);
        }
        published
    }

    #[test]
    fn a_bar_type_maps_to_the_symbol_and_interval_the_venue_echoes() {
        let key = feed_key(&bar_type(1, BarAggregation::Minute), Market::Perps).unwrap();

        assert_eq!(key, ("vBTC_vUSDC".to_string(), "1m".to_string()));
    }

    #[test]
    fn a_bar_specification_this_engine_cannot_serve_is_refused_at_subscribe_time() {
        // Accepting it would register a feed key the venue never pushes, leaving the strategy
        // waiting on a subscription that was never going to arrive.
        assert!(feed_key(&bar_type(3, BarAggregation::Minute), Market::Perps).is_err());
    }

    #[tokio::test]
    async fn a_closed_bar_is_published() {
        let subscribed = bar_type(1, BarAggregation::Minute);

        let published = publish(
            vec![SodexWsEvent::Candle(Box::new(candle(true)))],
            feeds_with(subscribed),
        )
        .await;

        assert_eq!(published.len(), 1);
        let DataEvent::Data(Data::Bar(bar)) = &published[0] else {
            panic!("expected a bar");
        };
        assert_eq!(bar.bar_type, subscribed);
        assert_eq!(bar.close.to_string(), "91976");
    }

    #[tokio::test]
    async fn a_bar_is_published_once_its_successor_starts() {
        // The venue was observed never to set the closed flag: the bar simply rolls. Waiting
        // for the flag would publish nothing at all, so a later open time is what proves the
        // held bar's window has elapsed.
        let subscribed = bar_type(1, BarAggregation::Minute);

        let published = publish(
            vec![
                SodexWsEvent::Candle(Box::new(candle_at(1_767_972_900_000, false))),
                SodexWsEvent::Candle(Box::new(candle_at(1_767_972_900_000, false))),
                SodexWsEvent::Candle(Box::new(candle_at(1_767_972_960_000, false))),
            ],
            feeds_with(subscribed),
        )
        .await;

        assert_eq!(published.len(), 1);
        let DataEvent::Data(Data::Bar(bar)) = &published[0] else {
            panic!("expected a bar");
        };
        // The first bar, not the one that is still forming.
        assert_eq!(bar.ts_event.as_u64(), 1_767_972_900_000 * 1_000_000);
    }

    #[tokio::test]
    async fn the_bar_still_forming_is_never_published() {
        // Two bars start, so only the first is complete; the second is still open when the
        // stream ends and must not be released by the ending itself.
        let published = publish(
            vec![
                SodexWsEvent::Candle(Box::new(candle_at(1_767_972_900_000, false))),
                SodexWsEvent::Candle(Box::new(candle_at(1_767_972_960_000, false))),
                SodexWsEvent::Candle(Box::new(candle_at(1_767_972_960_000, false))),
            ],
            feeds_with(bar_type(1, BarAggregation::Minute)),
        )
        .await;

        assert_eq!(published.len(), 1);
    }

    #[tokio::test]
    async fn a_bar_is_published_once_even_when_the_flag_arrives_after_the_roll() {
        // Belt and braces: if the venue starts setting the flag, a bar already released by
        // its successor must not be published a second time.
        let published = publish(
            vec![
                SodexWsEvent::Candle(Box::new(candle_at(1_767_972_900_000, false))),
                SodexWsEvent::Candle(Box::new(candle_at(1_767_972_960_000, false))),
                SodexWsEvent::Candle(Box::new(candle_at(1_767_972_960_000, true))),
            ],
            feeds_with(bar_type(1, BarAggregation::Minute)),
        )
        .await;

        assert_eq!(published.len(), 2);
        let opens: Vec<u64> = published
            .iter()
            .map(|event| {
                let DataEvent::Data(Data::Bar(bar)) = event else {
                    panic!("expected a bar");
                };
                bar.ts_event.as_u64() / 1_000_000
            })
            .collect();
        assert_eq!(opens, vec![1_767_972_900_000, 1_767_972_960_000]);
    }

    #[tokio::test]
    async fn an_out_of_order_push_does_not_republish_a_finished_bar() {
        // The venue republishes constantly; a late frame for an earlier bar must not rewrite
        // one the engine has already consumed.
        let published = publish(
            vec![
                SodexWsEvent::Candle(Box::new(candle_at(1_767_972_960_000, false))),
                SodexWsEvent::Candle(Box::new(candle_at(1_767_972_900_000, false))),
                SodexWsEvent::Candle(Box::new(candle_at(1_767_972_960_000, false))),
            ],
            feeds_with(bar_type(1, BarAggregation::Minute)),
        )
        .await;

        assert!(published.is_empty());
    }

    #[tokio::test]
    async fn a_reconnect_drops_the_held_bar_rather_than_completing_it_later() {
        // The gap means the held snapshot may not be that bar's final state. Publishing it
        // after the reconnect would present a partial bar as a complete one.
        let published = publish(
            vec![
                SodexWsEvent::Candle(Box::new(candle_at(1_767_972_900_000, false))),
                SodexWsEvent::Reconnected,
                SodexWsEvent::Candle(Box::new(candle_at(1_767_972_960_000, false))),
            ],
            feeds_with(bar_type(1, BarAggregation::Minute)),
        )
        .await;

        assert!(published.is_empty());
    }

    #[tokio::test]
    async fn a_forming_bar_is_not_published() {
        // The venue republishes the forming bar on every block. Publishing it would hand the
        // strategy an open, high, low and close that were not yet final at that timestamp,
        // and nothing yet proves this one is over.
        let published = publish(
            vec![SodexWsEvent::Candle(Box::new(candle(false)))],
            feeds_with(bar_type(1, BarAggregation::Minute)),
        )
        .await;

        assert!(published.is_empty());
    }

    #[tokio::test]
    async fn a_push_for_an_unsubscribed_feed_is_dropped() {
        // Rebuilding a bar type from the candle's own text would publish data for anything
        // the venue sent, including a feed nobody asked for.
        let published = publish(
            vec![SodexWsEvent::Candle(Box::new(candle(true)))],
            Arc::new(Mutex::new(HashMap::new())),
        )
        .await;

        assert!(published.is_empty());
    }

    #[tokio::test]
    async fn a_rejection_publishes_no_data_and_does_not_stop_the_stream() {
        let published = publish(
            vec![
                SodexWsEvent::RequestRejected {
                    op: crate::websocket::Op::Subscribe,
                    params: None,
                    reason: "unknown symbol".to_string(),
                },
                SodexWsEvent::Candle(Box::new(candle(true))),
            ],
            feeds_with(bar_type(1, BarAggregation::Minute)),
        )
        .await;

        assert_eq!(published.len(), 1);
    }
}
