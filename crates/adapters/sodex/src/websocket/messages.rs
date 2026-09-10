//! WebSocket wire protocol types.

use serde::{Deserialize, Serialize};

/// A client request. `op` selects the operation; `params` carries the channel selector.
///
/// `id` is echoed back in the acknowledgement, which is the only way to correlate an ack
/// with the request that produced it when several subscriptions are in flight.
#[derive(Debug, Clone, Serialize)]
pub struct WsRequest<P> {
    pub op: Op,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u64>,
    pub params: P,
}

/// Operations the client can send.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Op {
    Subscribe,
    Unsubscribe,
    Ping,
    Pong,
}

/// A bare `{"op":"ping"}` frame.
///
/// Serialized without `params`, which the venue's keepalive example omits.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Heartbeat {
    pub op: Op,
}

impl Heartbeat {
    #[must_use]
    pub const fn ping() -> Self {
        Self { op: Op::Ping }
    }
}

/// Acknowledgement of a subscribe or unsubscribe.
///
/// `success` is authoritative: a `false` carries the reason in `error`, and the echoed
/// `result` may be absent. `time_in`/`time_out` bracket the venue's own handling and are
/// useful for separating venue latency from network latency when diagnosing slow acks.
#[derive(Debug, Clone, Deserialize)]
pub struct WsAck<R> {
    pub op: Op,
    pub id: Option<u64>,
    pub result: Option<R>,
    pub success: bool,
    #[serde(rename = "connID")]
    pub conn_id: Option<String>,
    pub error: Option<String>,
    pub time_in: Option<u64>,
    pub time_out: Option<u64>,
}

impl<R> WsAck<R> {
    /// Converts the acknowledgement into a result.
    ///
    /// # Errors
    ///
    /// Returns the venue's error text when `success` is false, substituting a placeholder
    /// if the venue reported failure without a message.
    pub fn into_result(self) -> Result<Option<R>, String> {
        if self.success {
            Ok(self.result)
        } else {
            Err(self
                .error
                .unwrap_or_else(|| "venue reported failure without an error message".to_string()))
        }
    }
}

/// Subscription selector for the candle channel.
///
/// Doubles as the unsubscribe selector: the venue matches an unsubscribe against the
/// original params, so the same value must be reproducible field for field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CandleParams {
    pub channel: String,
    pub symbol: String,
    pub interval: String,
}

impl CandleParams {
    /// Channel name as the venue spells it.
    pub const CHANNEL: &'static str = "candle";

    #[must_use]
    pub fn new(symbol: impl Into<String>, interval: impl Into<String>) -> Self {
        Self {
            channel: Self::CHANNEL.to_string(),
            symbol: symbol.into(),
            interval: interval.into(),
        }
    }
}

/// A push frame carrying channel data.
#[derive(Debug, Clone, Deserialize)]
pub struct WsUpdate<T> {
    pub channel: String,
    #[serde(rename = "type")]
    pub update_type: String,
    pub data: T,
}

/// One OHLC bar.
///
/// Prices and sizes stay as strings all the way through this layer. The venue emits them as
/// decimal strings, and parsing them into a binary float here would discard precision before
/// anything has had a chance to convert them into the engine's exact numeric types.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Candle {
    /// Bar open time, milliseconds.
    #[serde(rename = "t")]
    pub open_time_ms: u64,
    /// Time this update was produced, milliseconds. Moves within an open bar.
    #[serde(rename = "T")]
    pub update_time_ms: u64,
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "i")]
    pub interval: String,
    #[serde(rename = "o")]
    pub open: String,
    #[serde(rename = "h")]
    pub high: String,
    #[serde(rename = "l")]
    pub low: String,
    #[serde(rename = "c")]
    pub close: String,
    /// Base-asset volume.
    #[serde(rename = "v")]
    pub volume: String,
    /// Quote-asset volume.
    #[serde(rename = "q")]
    pub quote_volume: String,
    /// Trades in the bar.
    #[serde(rename = "n")]
    pub trade_count: u64,
    /// Whether the bar has closed.
    ///
    /// **Only `true` bars are safe to trade on.** The channel republishes the forming bar on
    /// every block, so acting on `false` means deciding from an OHLC that can still move —
    /// a bar whose high, low and close are not yet final. Strategies driven by closed bars
    /// must filter on this flag; a backtest that used closed bars and a live run that did
    /// not are not comparable.
    #[serde(rename = "x")]
    pub closed: bool,
}

impl Candle {
    /// Whether this bar is final and therefore safe to act on.
    #[must_use]
    pub const fn is_final(&self) -> bool {
        self.closed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The venue's worked example, verbatim.
    const CANDLE_UPDATE: &str = r#"{
        "channel": "candle",
        "type": "update",
        "data": {
            "t": 1767972900000, "T": 1767972938724, "s": "BTC-USD", "i": "1m",
            "o": "91869", "h": "91982", "l": "91869", "c": "91976",
            "v": "4.12298", "q": "379148.6798", "n": 0, "x": false
        }
    }"#;

    #[test]
    fn parses_the_venue_candle_example() {
        let update: WsUpdate<Candle> = serde_json::from_str(CANDLE_UPDATE).unwrap();

        assert_eq!(update.channel, "candle");
        assert_eq!(update.update_type, "update");
        assert_eq!(update.data.symbol, "BTC-USD");
        assert_eq!(update.data.open_time_ms, 1_767_972_900_000);
        assert_eq!(update.data.update_time_ms, 1_767_972_938_724);
        assert_eq!(update.data.close, "91976");
        assert_eq!(update.data.trade_count, 0);
    }

    #[test]
    fn lowercase_t_and_uppercase_t_are_distinct_fields() {
        // The two timestamps differ only by case. Mapping both to one field would look
        // harmless and silently mislabel every bar's open time as its update time.
        let update: WsUpdate<Candle> = serde_json::from_str(CANDLE_UPDATE).unwrap();

        assert_ne!(update.data.open_time_ms, update.data.update_time_ms);
        assert!(update.data.update_time_ms > update.data.open_time_ms);
    }

    #[test]
    fn forming_bar_is_not_final() {
        let update: WsUpdate<Candle> = serde_json::from_str(CANDLE_UPDATE).unwrap();

        // The venue's own example is a forming bar. Anything consuming this stream has to
        // filter on the flag rather than assuming every push is a completed bar.
        assert!(!update.data.is_final());
    }

    #[test]
    fn closed_bar_is_final() {
        let raw = CANDLE_UPDATE.replace(r#""x": false"#, r#""x": true"#);
        let update: WsUpdate<Candle> = serde_json::from_str(&raw).unwrap();

        assert!(update.data.is_final());
    }

    #[test]
    fn prices_stay_as_strings() {
        let update: WsUpdate<Candle> = serde_json::from_str(CANDLE_UPDATE).unwrap();

        // Round-tripping through the struct must not reformat the decimal text; that would
        // mean a lossy numeric hop happened somewhere in between.
        assert_eq!(update.data.quote_volume, "379148.6798");
        let reserialized = serde_json::to_string(&update.data).unwrap();
        assert!(reserialized.contains(r#""q":"379148.6798""#), "{reserialized}");
    }

    #[test]
    fn subscribe_request_serializes_to_the_documented_shape() {
        let request = WsRequest {
            op: Op::Subscribe,
            id: None,
            params: CandleParams::new("BTC-USD", "1m"),
        };

        let json = serde_json::to_string(&request).unwrap();

        assert_eq!(
            json,
            r#"{"op":"subscribe","params":{"channel":"candle","symbol":"BTC-USD","interval":"1m"}}"#
        );
    }

    #[test]
    fn request_id_is_omitted_when_absent_and_present_when_set() {
        let without = serde_json::to_string(&WsRequest {
            op: Op::Unsubscribe,
            id: None,
            params: CandleParams::new("BTC-USD", "1m"),
        })
        .unwrap();
        assert!(!without.contains("\"id\""));

        let with = serde_json::to_string(&WsRequest {
            op: Op::Unsubscribe,
            id: Some(7),
            params: CandleParams::new("BTC-USD", "1m"),
        })
        .unwrap();
        assert!(with.contains(r#""id":7"#));
    }

    #[test]
    fn unsubscribe_params_match_subscribe_params_exactly() {
        // The venue identifies the feed to drop by matching the original params, so the two
        // must serialize identically. Divergence would leave a subscription alive.
        let subscribe = serde_json::to_value(CandleParams::new("BTC-USD", "1m")).unwrap();
        let unsubscribe = serde_json::to_value(CandleParams::new("BTC-USD", "1m")).unwrap();

        assert_eq!(subscribe, unsubscribe);
    }

    #[test]
    fn successful_ack_yields_the_echoed_selector() {
        let raw = r#"{
            "op":"subscribe",
            "result":{"channel":"candle","symbol":"BTC-USD","interval":"1m"},
            "success":true,
            "connID":"0xb0922dfe2b14dadb1195ea4db2b45508",
            "time_in":1672515782130,"time_out":1672515782136
        }"#;
        let ack: WsAck<CandleParams> = serde_json::from_str(raw).unwrap();

        assert_eq!(ack.conn_id.as_deref(), Some("0xb0922dfe2b14dadb1195ea4db2b45508"));
        assert_eq!(
            ack.into_result().unwrap(),
            Some(CandleParams::new("BTC-USD", "1m"))
        );
    }

    #[test]
    fn failed_ack_surfaces_the_error_text() {
        let raw = r#"{"op":"subscribe","result":null,"success":false,"error":"unknown symbol","time_in":1,"time_out":2}"#;
        let ack: WsAck<CandleParams> = serde_json::from_str(raw).unwrap();

        assert_eq!(ack.into_result().unwrap_err(), "unknown symbol");
    }

    #[test]
    fn failure_without_message_still_reports_an_error() {
        // Defensive: success=false with a null error must not be mistaken for success.
        let raw = r#"{"op":"subscribe","result":null,"success":false,"error":null,"time_in":1,"time_out":2}"#;
        let ack: WsAck<CandleParams> = serde_json::from_str(raw).unwrap();

        assert!(ack.into_result().is_err());
    }

    #[test]
    fn ping_frame_carries_no_params() {
        let json = serde_json::to_string(&Heartbeat::ping()).unwrap();

        assert_eq!(json, r#"{"op":"ping"}"#);
    }
}
