//! Connection lifecycle and subscription bookkeeping for the SoDEX streams.
//!
//! # The desired set is the only source of truth
//!
//! Every subscription this client has been asked for lives in one list, and every outbound
//! subscribe is bound to the connection epoch it was composed for. A send that loses the race
//! against a reconnect is therefore not an error: the replacement connection replays the whole
//! desired set anyway, so the lost message would have been a duplicate. The alternative —
//! letting the transport buffer and replay the send — would produce exactly that duplicate,
//! because the replay and the reconnect handler would both deliver it.
//!
//! The same list is what makes a reconnect survivable at all. The venue keeps no subscription
//! state across connections, so a client that does not re-send loses its feed silently: the
//! socket is up, the strategy simply never receives another bar.

use std::{collections::HashMap, sync::Arc};

use nautilus_network::{
    RECONNECTED,
    error::SendError,
    websocket::{WebSocketClient, WebSocketConfig, channel_epoch_message_handler},
};
use tokio::sync::{
    Mutex,
    mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel},
};
use tokio_tungstenite::tungstenite::Message;

use super::{
    DEFAULT_IDLE_PROBE_SECS,
    messages::{Candle, CandleParams, Heartbeat, Op, WsAck, WsRequest, WsUpdate},
    probe_interval_is_sound, stream_url,
};
use crate::{common::Market, http::Network};

/// Failure modes of the streaming client.
#[derive(Debug, thiserror::Error)]
pub enum WsError {
    #[error("web socket client is not connected")]
    NotConnected,
    #[error(
        "probe interval of {0}s leaves no room for a ping and its reply before the venue drops \
         an idle connection; it must stay under half the idle limit"
    )]
    ProbeIntervalTooLong(u64),
    #[error("serialization failed: {0}")]
    Serialize(#[from] serde_json::Error),
    #[error("transport failed: {0}")]
    Transport(String),
}

/// What the client hands to its consumer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SodexWsEvent {
    /// A bar from the candle channel. Forming bars are included; see [`Candle::is_final`].
    Candle(Box<Candle>),
    /// The venue refused a subscribe or unsubscribe.
    ///
    /// A refused subscribe means no data will arrive for that selector, which is otherwise
    /// indistinguishable from a quiet market.
    RequestRejected {
        op: Op,
        params: Option<CandleParams>,
        reason: String,
    },
    /// A replacement connection was established and the desired set re-sent.
    Reconnected,
}

/// An inbound frame, classified.
#[derive(Debug)]
enum Inbound {
    Reconnected,
    /// A keepalive frame in either direction. Carries no payload worth surfacing, but its
    /// arrival is itself the liveness signal the transport is watching for.
    Heartbeat,
    Update(Box<WsUpdate<Candle>>),
    Ack(Box<WsAck<CandleParams>>),
    /// Well-formed JSON this adapter has no handling for — a channel it never subscribed to,
    /// or a field the venue added. Ignored rather than treated as a protocol failure.
    Unrecognized,
}

/// A request awaiting its acknowledgement.
///
/// The venue echoes `result` only on success, so a failed ack identifies itself solely by the
/// `id` that was sent with the request. Without this map a rejection would arrive with no way
/// to say which selector it rejected.
#[derive(Debug, Clone)]
struct Pending {
    op: Op,
    params: CandleParams,
}

#[derive(Debug, Default)]
struct State {
    desired: Vec<CandleParams>,
    pending: HashMap<u64, Pending>,
    next_id: u64,
}

impl State {
    fn take_id(&mut self) -> u64 {
        self.next_id += 1;
        self.next_id
    }

    /// Records a selector as wanted, reporting whether it was not already.
    ///
    /// A repeat subscribe is not an error but must not reach the venue: the desired list would
    /// then hold one entry while the connection held two, and the replay after a reconnect
    /// would quietly halve the feed's duplication.
    fn want(&mut self, params: &CandleParams) -> bool {
        if self.desired.contains(params) {
            return false;
        }
        self.desired.push(params.clone());
        true
    }

    /// Drops a selector, reporting whether it was wanted.
    fn unwant(&mut self, params: &CandleParams) -> bool {
        let Some(index) = self.desired.iter().position(|existing| existing == params) else {
            return false;
        };
        self.desired.remove(index);
        true
    }
}

/// A live connection and the task draining it.
#[derive(Debug)]
struct Connection {
    client: Arc<WebSocketClient>,
    reader: tokio::task::JoinHandle<()>,
}

/// Streaming client for one network and market.
///
/// Subscribing takes `&self` so the client can be shared: a data client hands one to the task
/// that drains the stream and still subscribes from the engine's thread. The connection is
/// therefore held behind a lock rather than in `&mut self`.
#[derive(Debug)]
pub struct SodexWebSocketClient {
    url: String,
    probe_interval_secs: u64,
    connection: parking_lot::Mutex<Option<Connection>>,
    state: Arc<Mutex<State>>,
}

impl SodexWebSocketClient {
    /// Creates a client for one network and market, using the default probe interval.
    #[must_use]
    pub fn new(network: Network, market: Market) -> Self {
        Self {
            url: stream_url(network, market),
            probe_interval_secs: DEFAULT_IDLE_PROBE_SECS,
            connection: parking_lot::Mutex::new(None),
            state: Arc::new(Mutex::new(State::default())),
        }
    }

    /// Overrides how long the client waits before probing an idle link.
    ///
    /// # Errors
    ///
    /// Returns [`WsError::ProbeIntervalTooLong`] when the interval leaves no room to detect a
    /// dead link before the venue closes it.
    pub fn with_probe_interval_secs(mut self, probe_secs: u64) -> Result<Self, WsError> {
        if !probe_interval_is_sound(probe_secs) {
            return Err(WsError::ProbeIntervalTooLong(probe_secs));
        }
        self.probe_interval_secs = probe_secs;
        Ok(self)
    }

    /// The transport configuration this client connects with.
    fn config(&self) -> WebSocketConfig {
        WebSocketConfig {
            url: self.url.clone(),
            headers: Vec::new(),
            heartbeat_interval_secs: Some(self.probe_interval_secs),
            // A text frame, not an empty Ping control frame: this venue counts an
            // application-level keepalive, and the two are not interchangeable.
            heartbeat_payload: Some(
                serde_json::to_string(&Heartbeat::ping())
                    .expect("a two-field struct of plain values cannot fail to serialize"),
            ),
            connect_timeout_ms: None,
            reconnect_delay_initial_ms: None,
            reconnect_delay_max_ms: None,
            reconnect_backoff_factor: None,
            reconnect_jitter_ms: None,
            reconnect_max_attempts: None,
            // Two probe intervals, which is one ping plus the wait for its reply, and still
            // inside the venue's idle limit.
            heartbeat_timeout_secs: Some(self.probe_interval_secs * 2),
            // Deliberately unset. The venue answers the keepalive with a text frame, which
            // refreshes this timer exactly as real data would, so it could not distinguish a
            // stalled feed from a healthy one here.
            idle_timeout_ms: None,
            backend: nautilus_network::websocket::TransportBackend::default(),
            proxy_url: None,
        }
    }

    /// Connects and starts consuming the stream.
    ///
    /// # Errors
    ///
    /// Returns [`WsError::Transport`] if the connection cannot be established.
    pub async fn connect(&self) -> Result<UnboundedReceiver<SodexWsEvent>, WsError> {
        let (message_handler, raw_rx) = channel_epoch_message_handler();

        let client = WebSocketClient::epoch_builder()
            .config(self.config())
            .epoch_handler(message_handler)
            .connect()
            .await
            .map_err(|error| WsError::Transport(error.to_string()))?;
        let client = Arc::new(client);

        let (events_tx, events_rx) = unbounded_channel();
        let reader = tokio::spawn(read_stream(
            raw_rx,
            events_tx,
            Arc::clone(&client),
            Arc::clone(&self.state),
        ));

        if let Some(previous) = self.connection.lock().replace(Connection { client, reader }) {
            previous.reader.abort();
        }

        Ok(events_rx)
    }

    /// The live transport, if this client has connected.
    ///
    /// Cloned out of the lock rather than borrowed: sends await, and holding a
    /// non-async lock across an await would block every other caller on the socket.
    fn transport(&self) -> Result<Arc<WebSocketClient>, WsError> {
        self.connection
            .lock()
            .as_ref()
            .map(|connection| Arc::clone(&connection.client))
            .ok_or(WsError::NotConnected)
    }

    /// Whether this client has a live connection.
    #[must_use]
    pub fn is_connected(&self) -> bool {
        self.connection
            .lock()
            .as_ref()
            .is_some_and(|connection| connection.client.is_active())
    }

    /// Subscribes to a candle feed, doing nothing if it is already subscribed.
    ///
    /// # Errors
    ///
    /// Returns [`WsError::NotConnected`] before [`Self::connect`], or [`WsError::Transport`]
    /// if the send fails for a reason a reconnect will not repair.
    pub async fn subscribe_candles(&self, params: CandleParams) -> Result<(), WsError> {
        let client = self.transport()?;
        let mut state = self.state.lock().await;

        if !state.want(&params) {
            return Ok(());
        }

        // The epoch is read under the same lock the reconnect handler takes, so this send is
        // either composed for the connection the handler has already replayed onto, or it is
        // in the desired list before that replay reads it. There is no ordering in which the
        // subscription is both absent from the replay and bound to a dead connection.
        send_request(&client, &mut state, Op::Subscribe, params).await
    }

    /// Unsubscribes from a candle feed, doing nothing if it is not subscribed.
    ///
    /// # Errors
    ///
    /// Returns [`WsError::NotConnected`] before [`Self::connect`], or [`WsError::Transport`]
    /// if the send fails for a reason a reconnect will not repair.
    pub async fn unsubscribe_candles(&self, params: &CandleParams) -> Result<(), WsError> {
        let client = self.transport()?;
        let mut state = self.state.lock().await;

        if !state.unwant(params) {
            return Ok(());
        }

        // Losing this send to a reconnect needs no repair: the replacement connection starts
        // with no subscriptions and is replayed from a desired list that no longer holds this
        // one, so the feed is gone either way.
        send_request(&client, &mut state, Op::Unsubscribe, params.clone()).await
    }

    /// The selectors this client currently wants subscribed.
    pub async fn subscriptions(&self) -> Vec<CandleParams> {
        self.state.lock().await.desired.clone()
    }

    /// Closes the connection and stops consuming the stream.
    pub async fn close(&self) {
        let Some(connection) = self.connection.lock().take() else {
            return;
        };
        connection.client.disconnect().await;
        connection.reader.abort();
    }
}

/// Composes a request, records it for acknowledgement, and binds it to the live connection.
async fn send_request(
    client: &WebSocketClient,
    state: &mut State,
    op: Op,
    params: CandleParams,
) -> Result<(), WsError> {
    let epoch = client.connection_epoch();
    let id = state.take_id();
    state.pending.insert(
        id,
        Pending {
            op,
            params: params.clone(),
        },
    );

    let text = serde_json::to_string(&WsRequest {
        op,
        id: Some(id),
        params,
    })?;

    match client.send_text_on_connection(text, None, epoch).await {
        Ok(()) => Ok(()),
        // The connection this was composed for is gone, and so is the subscription state it
        // held. The reconnect handler resends from the desired list, which already reflects
        // this call, so there is nothing to repair and nothing to report.
        Err(SendError::ConnectionChanged) => {
            state.pending.remove(&id);
            Ok(())
        }
        Err(error) => {
            state.pending.remove(&id);
            Err(WsError::Transport(error.to_string()))
        }
    }
}

/// Consumes the raw stream until it ends or the consumer goes away.
async fn read_stream(
    mut raw_rx: UnboundedReceiver<(u64, Message)>,
    events_tx: UnboundedSender<SodexWsEvent>,
    client: Arc<WebSocketClient>,
    state: Arc<Mutex<State>>,
) {
    while let Some((epoch, message)) = raw_rx.recv().await {
        let Message::Text(text) = message else {
            // Binary and control frames carry nothing this protocol uses. The transport
            // already answers Ping frames and counts every frame towards liveness.
            continue;
        };

        let event = match classify(&text) {
            Ok(Inbound::Reconnected) => {
                replay_subscriptions(&client, &state, epoch).await;
                Some(SodexWsEvent::Reconnected)
            }
            Ok(Inbound::Heartbeat) | Ok(Inbound::Unrecognized) => None,
            Ok(Inbound::Update(update)) => Some(SodexWsEvent::Candle(Box::new(update.data))),
            Ok(Inbound::Ack(ack)) => resolve_ack(&state, *ack).await,
            Err(reason) => {
                log::warn!("sodex_ws_frame_unparsed reason={reason}");
                None
            }
        };

        if let Some(event) = event
            && events_tx.send(event).is_err()
        {
            log::debug!("sodex_ws_consumer_gone");
            break;
        }
    }
}

/// Re-sends the desired set onto a replacement connection.
///
/// The venue keeps no subscription state across connections, so without this a reconnect
/// leaves a live socket carrying nothing.
async fn replay_subscriptions(client: &WebSocketClient, state: &Arc<Mutex<State>>, epoch: u64) {
    let mut state = state.lock().await;

    // Acknowledgements owed by the previous connection will never arrive. Leaving them would
    // let a rejection on the new connection be matched against a stale entry and reported
    // against the wrong selector.
    state.pending.clear();

    for params in state.desired.clone() {
        let id = state.take_id();
        state.pending.insert(
            id,
            Pending {
                op: Op::Subscribe,
                params: params.clone(),
            },
        );

        let text = match serde_json::to_string(&WsRequest {
            op: Op::Subscribe,
            id: Some(id),
            params,
        }) {
            Ok(text) => text,
            Err(error) => {
                log::error!("sodex_ws_resubscribe_unserializable error={error}");
                state.pending.remove(&id);
                continue;
            }
        };

        match client.send_text_on_connection(text, None, epoch).await {
            Ok(()) => {}
            // A newer connection already replaced this one, and its own replay covers the
            // whole desired set. Continuing here would only compose messages for a dead epoch.
            Err(SendError::ConnectionChanged) => {
                state.pending.remove(&id);
                break;
            }
            // The selector stays in the desired list, so the next reconnect retries it. Until
            // then this feed is silent, which is why this is logged at error level rather than
            // being left to look like a quiet market.
            Err(error) => {
                log::error!("sodex_ws_resubscribe_failed epoch={epoch} error={error}");
                state.pending.remove(&id);
            }
        }
    }
}

/// Matches an acknowledgement to the request that produced it.
async fn resolve_ack(
    state: &Arc<Mutex<State>>,
    ack: WsAck<CandleParams>,
) -> Option<SodexWsEvent> {
    let pending = match ack.id {
        Some(id) => state.lock().await.pending.remove(&id),
        None => None,
    };

    match ack.into_result() {
        Ok(_) => None,
        Err(reason) => Some(SodexWsEvent::RequestRejected {
            op: pending.as_ref().map_or(Op::Subscribe, |entry| entry.op),
            params: pending.map(|entry| entry.params),
            reason,
        }),
    }
}

/// Sorts an inbound text frame into the shapes this adapter handles.
///
/// # Errors
///
/// Returns the parse failure when a frame announces a shape this adapter knows but does not
/// match it. A frame announcing an unknown shape is [`Inbound::Unrecognized`], not an error.
fn classify(text: &str) -> Result<Inbound, String> {
    if text == RECONNECTED {
        return Ok(Inbound::Reconnected);
    }

    let value: serde_json::Value =
        serde_json::from_str(text).map_err(|error| error.to_string())?;

    // Acknowledgements carry `op`; channel pushes carry `channel` and `type`.
    if let Some(op) = value.get("op").and_then(serde_json::Value::as_str) {
        return match op {
            "ping" | "pong" => Ok(Inbound::Heartbeat),
            "subscribe" | "unsubscribe" => serde_json::from_value(value)
                .map(|ack| Inbound::Ack(Box::new(ack)))
                .map_err(|error| error.to_string()),
            _ => Ok(Inbound::Unrecognized),
        };
    }

    if value.get("channel").and_then(serde_json::Value::as_str) == Some(CandleParams::CHANNEL) {
        return serde_json::from_value(value)
            .map(|update| Inbound::Update(Box::new(update)))
            .map_err(|error| error.to_string());
    }

    Ok(Inbound::Unrecognized)
}

#[cfg(test)]
mod tests {
    use super::{super::SERVER_IDLE_DISCONNECT_SECS, *};

    fn params() -> CandleParams {
        CandleParams::new("BTC-USD", "1m")
    }

    fn client() -> SodexWebSocketClient {
        SodexWebSocketClient::new(Network::Testnet, Market::Perps)
    }

    #[test]
    fn the_keepalive_is_a_text_frame_not_a_control_ping() {
        // An empty Ping control frame does not count as activity on this venue, so leaving the
        // payload unset would keep the transport happy while the venue hung up on schedule.
        let config = client().config();

        assert_eq!(config.heartbeat_payload.as_deref(), Some(r#"{"op":"ping"}"#));
    }

    #[test]
    fn the_dead_link_verdict_lands_before_the_venue_hangs_up() {
        let config = client().config();

        let probe = config.heartbeat_interval_secs.unwrap();
        let verdict = config.heartbeat_timeout_secs.unwrap();

        assert_eq!(verdict, probe * 2);
        assert!(verdict < SERVER_IDLE_DISCONNECT_SECS);
    }

    #[test]
    fn the_idle_timer_is_left_off_because_the_venue_answers_in_text() {
        // The venue's reply to the keepalive is a text frame, which refreshes the idle timer
        // exactly as a bar would. Setting it here would produce a check that can never fire.
        assert!(client().config().idle_timeout_ms.is_none());
    }

    #[test]
    fn a_probe_interval_that_cannot_detect_failure_in_time_is_refused() {
        let error = client().with_probe_interval_secs(30).unwrap_err();

        assert!(matches!(error, WsError::ProbeIntervalTooLong(30)));
    }

    #[test]
    fn a_sound_probe_interval_reaches_the_transport_configuration() {
        let config = client().with_probe_interval_secs(5).unwrap().config();

        assert_eq!(config.heartbeat_interval_secs, Some(5));
        assert_eq!(config.heartbeat_timeout_secs, Some(10));
    }

    #[test]
    fn wanting_the_same_selector_twice_sends_only_once() {
        let mut state = State::default();

        assert!(state.want(&params()));
        assert!(!state.want(&params()));
        assert_eq!(state.desired, vec![params()]);
    }

    #[test]
    fn unwanting_an_absent_selector_reports_nothing_to_do() {
        let mut state = State::default();

        assert!(!state.unwant(&params()));

        state.want(&params());
        assert!(state.unwant(&params()));
        assert!(state.desired.is_empty());
    }

    #[test]
    fn request_ids_are_never_reused() {
        let mut state = State::default();

        let first = state.take_id();
        let second = state.take_id();

        assert_ne!(first, second);
    }

    #[test]
    fn the_reconnect_sentinel_is_recognised_before_any_parsing() {
        // It is not JSON, so a classifier that parsed first would report it as a malformed
        // frame and the subscriptions would never be replayed.
        assert!(matches!(classify(RECONNECTED), Ok(Inbound::Reconnected)));
    }

    #[test]
    fn keepalive_replies_are_classified_as_heartbeats() {
        assert!(matches!(classify(r#"{"op":"pong"}"#), Ok(Inbound::Heartbeat)));
        assert!(matches!(classify(r#"{"op":"ping"}"#), Ok(Inbound::Heartbeat)));
    }

    #[test]
    fn a_candle_push_is_classified_as_an_update() {
        let raw = r#"{"channel":"candle","type":"update","data":{
            "t":1767972900000,"T":1767972938724,"s":"BTC-USD","i":"1m",
            "o":"91869","h":"91982","l":"91869","c":"91976",
            "v":"4.12298","q":"379148.6798","n":0,"x":false}}"#;

        let Ok(Inbound::Update(update)) = classify(raw) else {
            panic!("expected a candle update");
        };
        assert_eq!(update.data.symbol, "BTC-USD");
    }

    #[test]
    fn an_acknowledgement_is_told_apart_from_a_push_by_its_op_field() {
        let raw = r#"{"op":"subscribe","id":4,"result":null,"success":true,"time_in":1,"time_out":2}"#;

        let Ok(Inbound::Ack(ack)) = classify(raw) else {
            panic!("expected an acknowledgement");
        };
        assert_eq!(ack.id, Some(4));
    }

    #[test]
    fn an_unknown_channel_or_op_is_ignored_rather_than_failing_the_stream() {
        // The venue may add channels this adapter never subscribed to. Treating those as
        // protocol errors would turn a harmless addition into a flood of warnings.
        assert!(matches!(
            classify(r#"{"channel":"trades","type":"update","data":{}}"#),
            Ok(Inbound::Unrecognized)
        ));
        assert!(matches!(
            classify(r#"{"op":"auth","success":true}"#),
            Ok(Inbound::Unrecognized)
        ));
    }

    #[test]
    fn a_candle_push_missing_a_field_is_reported_not_swallowed() {
        // Announcing the candle channel and then not matching its shape is a real mismatch
        // between this adapter and the venue, and must not look like an unknown channel.
        let raw = r#"{"channel":"candle","type":"update","data":{"s":"BTC-USD"}}"#;

        assert!(classify(raw).is_err());
    }

    #[tokio::test]
    async fn a_rejection_names_the_selector_that_was_refused() {
        // The venue echoes the selector only on success, so the id recorded when the request
        // went out is the only thing that can name what was refused.
        let state = Arc::new(Mutex::new(State::default()));
        state.lock().await.pending.insert(
            9,
            Pending {
                op: Op::Subscribe,
                params: params(),
            },
        );

        let ack: WsAck<CandleParams> = serde_json::from_str(
            r#"{"op":"subscribe","id":9,"result":null,"success":false,"error":"unknown symbol","time_in":1,"time_out":2}"#,
        )
        .unwrap();

        let event = resolve_ack(&state, ack).await.unwrap();

        assert_eq!(
            event,
            SodexWsEvent::RequestRejected {
                op: Op::Subscribe,
                params: Some(params()),
                reason: "unknown symbol".to_string(),
            }
        );
        assert!(state.lock().await.pending.is_empty());
    }

    #[tokio::test]
    async fn a_successful_acknowledgement_produces_no_event() {
        let state = Arc::new(Mutex::new(State::default()));
        state.lock().await.pending.insert(
            1,
            Pending {
                op: Op::Subscribe,
                params: params(),
            },
        );

        let ack: WsAck<CandleParams> = serde_json::from_str(
            r#"{"op":"subscribe","id":1,"result":{"channel":"candle","symbol":"BTC-USD","interval":"1m"},"success":true,"time_in":1,"time_out":2}"#,
        )
        .unwrap();

        assert!(resolve_ack(&state, ack).await.is_none());
        assert!(state.lock().await.pending.is_empty());
    }

    #[tokio::test]
    async fn a_rejection_with_no_id_still_surfaces() {
        // Losing the selector is bad; losing the rejection entirely would be worse, because
        // the missing feed would then be indistinguishable from a quiet market.
        let state = Arc::new(Mutex::new(State::default()));

        let ack: WsAck<CandleParams> = serde_json::from_str(
            r#"{"op":"subscribe","result":null,"success":false,"error":"rate limited","time_in":1,"time_out":2}"#,
        )
        .unwrap();

        let event = resolve_ack(&state, ack).await.unwrap();

        assert!(matches!(
            event,
            SodexWsEvent::RequestRejected { params: None, .. }
        ));
    }
}
