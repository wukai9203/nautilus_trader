//! WebSocket streams for SoDEX.
//!
//! # Streams carry no authorization
//!
//! The venue states that user-specific streams require no subscription authorization and
//! that any client may subscribe to another user's data. Two consequences worth stating
//! plainly:
//!
//! - Consuming account streams needs no API key, which simplifies this layer.
//! - Account activity is **not private**. Anyone holding an account id can watch that
//!   account's orders, fills and balance changes in real time. A strategy whose behaviour is
//!   inferable from its order flow — laddered entries, fixed grid spacing, predictable
//!   scale-ins — is exposed by merely trading here.
//!
//! # Liveness
//!
//! The venue drops a connection after 60 seconds without a subscription or pushed data, so
//! silence is indistinguishable from a dead link and must be probed. [`Keepalive`] implements
//! the documented protocol: reset on any inbound frame, ping once idle, reconnect if the pong
//! does not arrive.

pub mod messages;

pub use messages::{Candle, CandleParams, Heartbeat, Op, WsAck, WsRequest, WsUpdate};

use crate::{common::Market, http::Network};

/// How long the venue tolerates silence before dropping the connection.
pub const SERVER_IDLE_DISCONNECT_MS: u64 = 60_000;

/// Default idle period before probing with a ping.
///
/// The full detection cycle costs two of these — one waiting for traffic, one waiting for the
/// pong — so the value must stay under half of [`SERVER_IDLE_DISCONNECT_MS`]. Otherwise the
/// venue would close the connection before this side concluded anything was wrong, turning
/// an orderly reconnect into a surprise disconnect.
pub const DEFAULT_IDLE_PROBE_MS: u64 = 20_000;

/// WebSocket URL for one network and market.
#[must_use]
pub fn stream_url(network: Network, market: Market) -> String {
    format!("{}/{}", network.ws_base(), market.path_segment())
}

/// What the connection should do next.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeepaliveAction {
    /// Traffic is recent enough; do nothing.
    Idle,
    /// Silent for a full probe period; send a ping.
    SendPing,
    /// A ping went unanswered for a full probe period; the link is presumed dead.
    Reconnect,
}

/// Liveness tracker for one connection.
///
/// Time is supplied by the caller rather than read from a clock, so the whole cycle —
/// including the reconnect path, which is otherwise awkward to reach — is exercisable in
/// tests without sleeping.
#[derive(Debug)]
pub struct Keepalive {
    probe_after_ms: u64,
    last_inbound_ms: u64,
    ping_sent_at_ms: Option<u64>,
}

impl Keepalive {
    /// Creates a tracker using [`DEFAULT_IDLE_PROBE_MS`].
    #[must_use]
    pub fn new(now_ms: u64) -> Self {
        Self::with_probe_interval(now_ms, DEFAULT_IDLE_PROBE_MS)
    }

    /// Creates a tracker with an explicit probe interval.
    ///
    /// # Panics
    ///
    /// Panics if the interval is not under half the venue's disconnect threshold, since such
    /// a tracker could never reach [`KeepaliveAction::Reconnect`] before the venue hung up —
    /// a misconfiguration that would otherwise surface only as unexplained disconnects.
    #[must_use]
    pub fn with_probe_interval(now_ms: u64, probe_after_ms: u64) -> Self {
        assert!(
            probe_after_ms > 0 && probe_after_ms * 2 < SERVER_IDLE_DISCONNECT_MS,
            "probe interval {probe_after_ms}ms leaves no room for a ping/pong cycle within \
             the venue's {SERVER_IDLE_DISCONNECT_MS}ms disconnect window"
        );
        Self {
            probe_after_ms,
            last_inbound_ms: now_ms,
            ping_sent_at_ms: None,
        }
    }

    /// Records that a frame arrived.
    ///
    /// Any inbound frame counts, not just a pong: the venue's threshold is about traffic, so
    /// a busy market feed keeps the link alive without any pings at all.
    pub fn on_inbound(&mut self, now_ms: u64) {
        self.last_inbound_ms = now_ms;
        self.ping_sent_at_ms = None;
    }

    /// Records that a ping was sent, starting the pong deadline.
    pub fn on_ping_sent(&mut self, now_ms: u64) {
        self.ping_sent_at_ms = Some(now_ms);
    }

    /// Decides what the connection should do at `now_ms`.
    pub fn poll(&self, now_ms: u64) -> KeepaliveAction {
        if let Some(sent_at) = self.ping_sent_at_ms {
            return if now_ms.saturating_sub(sent_at) >= self.probe_after_ms {
                KeepaliveAction::Reconnect
            } else {
                KeepaliveAction::Idle
            };
        }

        if now_ms.saturating_sub(self.last_inbound_ms) >= self.probe_after_ms {
            KeepaliveAction::SendPing
        } else {
            KeepaliveAction::Idle
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PROBE: u64 = 20_000;

    fn tracker() -> Keepalive {
        Keepalive::with_probe_interval(0, PROBE)
    }

    #[test]
    fn stays_idle_while_traffic_is_recent() {
        let keepalive = tracker();

        assert_eq!(keepalive.poll(PROBE - 1), KeepaliveAction::Idle);
    }

    #[test]
    fn pings_once_silence_reaches_the_probe_interval() {
        let keepalive = tracker();

        assert_eq!(keepalive.poll(PROBE), KeepaliveAction::SendPing);
    }

    #[test]
    fn any_inbound_frame_resets_the_timer() {
        // Not just pongs: a busy market feed should keep the link alive with no pings.
        let mut keepalive = tracker();
        assert_eq!(keepalive.poll(PROBE), KeepaliveAction::SendPing);

        keepalive.on_inbound(PROBE);

        assert_eq!(keepalive.poll(PROBE + PROBE - 1), KeepaliveAction::Idle);
    }

    #[test]
    fn waits_quietly_while_a_pong_is_outstanding() {
        let mut keepalive = tracker();
        keepalive.on_ping_sent(PROBE);

        // Re-pinging into a silent link would just queue frames at a dead socket.
        assert_eq!(keepalive.poll(PROBE + PROBE - 1), KeepaliveAction::Idle);
    }

    #[test]
    fn reconnects_when_the_pong_never_arrives() {
        let mut keepalive = tracker();
        keepalive.on_ping_sent(PROBE);

        assert_eq!(keepalive.poll(PROBE * 2), KeepaliveAction::Reconnect);
    }

    #[test]
    fn a_pong_clears_the_deadline() {
        let mut keepalive = tracker();
        keepalive.on_ping_sent(PROBE);
        keepalive.on_inbound(PROBE + 100);

        assert_eq!(keepalive.poll(PROBE * 2), KeepaliveAction::Idle);
    }

    #[test]
    fn full_detection_cycle_completes_before_the_venue_hangs_up() {
        // The whole point of the interval bound: silence at t=0, ping at t=PROBE, reconnect
        // at t=2*PROBE — all inside the venue's 60s window.
        let mut keepalive = tracker();

        assert_eq!(keepalive.poll(PROBE), KeepaliveAction::SendPing);
        keepalive.on_ping_sent(PROBE);
        assert_eq!(keepalive.poll(PROBE * 2), KeepaliveAction::Reconnect);

        assert!(PROBE * 2 < SERVER_IDLE_DISCONNECT_MS);
    }

    #[test]
    #[should_panic(expected = "leaves no room for a ping/pong cycle")]
    fn rejects_a_probe_interval_that_cannot_detect_failure_in_time() {
        // 30s would mean concluding "dead" at 60s — exactly when the venue drops the link,
        // so the reconnect would always be reactive rather than pre-emptive.
        let _ = Keepalive::with_probe_interval(0, 30_000);
    }

    #[test]
    fn stream_urls_pair_network_with_market() {
        assert_eq!(
            stream_url(Network::Testnet, Market::Perps),
            "wss://testnet-gw.sodex.dev/ws/perps"
        );
        assert_eq!(
            stream_url(Network::Mainnet, Market::Spot),
            "wss://mainnet-gw.sodex.dev/ws/spot"
        );
    }
}
