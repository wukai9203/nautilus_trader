//! Request weight accounting.
//!
//! SoDEX limits traffic along three axes that are counted separately, and a client that
//! models only the first will still get rejected by the other two:
//!
//! 1. **IP weight** — 1200 per rolling minute, per IP, across every REST endpoint.
//! 2. **Order placement** — 600 orders/minute and 20 orders/second per account for API-key
//!    clients. Counted in orders, not requests.
//! 3. **Address limits** — per user address, applying to actions only (never to queries).
//!    The allowance accrues at one request per 1 USDC traded cumulatively since the address
//!    was created, starting from a buffer of 10,000 requests. Cancels get a higher ceiling
//!    (`min(limit + 100_000, limit * 2)`) so open orders can always be wound down.
//!
//! A batch is where the axes visibly disagree: `N` orders in one request cost **one**
//! request's weight against the IP budget but count as **N** against the address limit.
//! [`BatchCost`] exists so that difference cannot be silently collapsed into a single
//! number at a call site.
//!
//! Axis 3 depends on cumulative traded volume, which only the venue knows, so this module
//! models axes 1 and 2 and leaves axis 3 to be enforced by the venue's rejection.

use std::collections::VecDeque;

/// Weight allowed per rolling window, per IP.
pub const WEIGHT_PER_MINUTE: u32 = 1200;

/// Length of the rolling weight window.
pub const WEIGHT_WINDOW_MS: u64 = 60_000;

/// Weight charged to any endpoint not named in the venue's table.
pub const DEFAULT_ENDPOINT_WEIGHT: u32 = 20;

/// Orders per minute permitted to an API-key client, per account.
pub const ORDERS_PER_MINUTE: u32 = 600;

/// Orders per second permitted to an API-key client, per account.
pub const ORDERS_PER_SECOND: u32 = 20;

/// Raised when a request would exceed a budget. Carries how long to wait rather than a bare
/// failure, so callers can back off precisely instead of guessing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("{axis} exhausted: need {needed}, {available} available, retry in {retry_after_ms}ms")]
pub struct RateLimited {
    pub axis: Axis,
    pub needed: u32,
    pub available: u32,
    pub retry_after_ms: u64,
}

/// Which budget rejected a request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    IpWeight,
    OrdersPerMinute,
    OrdersPerSecond,
}

impl std::fmt::Display for Axis {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::IpWeight => "IP weight budget",
            Self::OrdersPerMinute => "orders-per-minute budget",
            Self::OrdersPerSecond => "orders-per-second budget",
        };
        f.write_str(name)
    }
}

/// What one batched request costs, split by axis.
///
/// The two fields are deliberately different units — weight versus order count — because
/// the venue counts them that way.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BatchCost {
    /// Charged against the per-IP weight budget.
    pub ip_weight: u32,
    /// Charged against the per-account order-placement limits.
    pub order_count: u32,
}

impl BatchCost {
    /// Cost of submitting, cancelling, or replacing `batch_size` orders in one request.
    #[must_use]
    pub const fn for_batch(batch_size: u32) -> Self {
        Self {
            ip_weight: batch_weight(batch_size),
            order_count: batch_size,
        }
    }
}

/// Weight of a batched order request: `1 + floor(N / 40)`.
#[must_use]
pub const fn batch_weight(batch_size: u32) -> u32 {
    1 + batch_size / 40
}

/// Weight of an order-book query, which the venue scales by requested depth.
#[must_use]
pub const fn orderbook_weight(depth: u32) -> u32 {
    match depth {
        0..=100 => 5,
        101..=500 => 10,
        _ => 20,
    }
}

/// Extra weight charged *after* a history response, based on rows returned:
/// `floor(items / 20)`.
///
/// This lands after the fact, so it is recorded with [`WeightBudget::record`] rather than
/// reserved up front — the count is not knowable before the response arrives.
#[must_use]
pub const fn history_extra_weight(items_returned: u32) -> u32 {
    items_returned / 20
}

/// Extra weight for a kline response: `max(1, floor(rows / 25))`.
///
/// Note this never reaches zero, unlike [`history_extra_weight`].
#[must_use]
pub const fn kline_extra_weight(rows_returned: u32) -> u32 {
    let scaled = rows_returned / 25;
    if scaled > 1 { scaled } else { 1 }
}

/// Rolling-window accounting for the per-IP weight budget.
///
/// Time is passed in rather than read from a clock so the window can be exercised
/// deterministically in tests.
#[derive(Debug)]
pub struct WeightBudget {
    limit: u32,
    window_ms: u64,
    entries: VecDeque<(u64, u32)>,
    consumed: u32,
}

impl WeightBudget {
    /// A budget with the venue's documented limits.
    #[must_use]
    pub fn new() -> Self {
        Self::with_limit(WEIGHT_PER_MINUTE, WEIGHT_WINDOW_MS)
    }

    /// A budget with explicit limits, for tests and for tightening below the venue's cap.
    #[must_use]
    pub fn with_limit(limit: u32, window_ms: u64) -> Self {
        Self {
            limit,
            window_ms,
            entries: VecDeque::new(),
            consumed: 0,
        }
    }

    /// Reserves `weight` if the window has room, otherwise reports how long to wait.
    ///
    /// # Errors
    ///
    /// Returns [`RateLimited`] when the reservation would exceed the window's limit.
    pub fn try_consume(&mut self, weight: u32, now_ms: u64) -> Result<(), RateLimited> {
        self.expire(now_ms);

        // `consumed` can exceed `limit` because after-the-fact charges are booked
        // unconditionally, so both arms here must be saturating.
        if self.consumed.saturating_add(weight) > self.limit {
            return Err(RateLimited {
                axis: Axis::IpWeight,
                needed: weight,
                available: self.limit.saturating_sub(self.consumed),
                retry_after_ms: self.retry_after(now_ms),
            });
        }

        self.record(weight, now_ms);
        Ok(())
    }

    /// Books weight unconditionally.
    ///
    /// For charges the venue applies after the fact — history and kline extras — where
    /// refusing is not an option because the request already happened.
    pub fn record(&mut self, weight: u32, now_ms: u64) {
        self.expire(now_ms);
        self.entries.push_back((now_ms, weight));
        self.consumed += weight;
    }

    /// Weight still available in the current window.
    pub fn available(&mut self, now_ms: u64) -> u32 {
        self.expire(now_ms);
        self.limit.saturating_sub(self.consumed)
    }

    fn expire(&mut self, now_ms: u64) {
        let cutoff = now_ms.saturating_sub(self.window_ms);
        while let Some(&(at, weight)) = self.entries.front() {
            if at > cutoff {
                break;
            }
            self.entries.pop_front();
            self.consumed -= weight;
        }
    }

    fn retry_after(&self, now_ms: u64) -> u64 {
        self.entries
            .front()
            .map_or(0, |&(at, _)| (at + self.window_ms).saturating_sub(now_ms))
    }
}

impl Default for WeightBudget {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn batch_weight_matches_the_documented_table() {
        // The venue publishes these three brackets explicitly.
        assert_eq!(batch_weight(1), 1);
        assert_eq!(batch_weight(39), 1);
        assert_eq!(batch_weight(40), 2);
        assert_eq!(batch_weight(79), 2);
        assert_eq!(batch_weight(80), 3);
        assert_eq!(batch_weight(119), 3);
    }

    #[test]
    fn orderbook_weight_follows_depth_brackets() {
        assert_eq!(orderbook_weight(1), 5);
        assert_eq!(orderbook_weight(100), 5);
        assert_eq!(orderbook_weight(101), 10);
        assert_eq!(orderbook_weight(500), 10);
        assert_eq!(orderbook_weight(501), 20);
    }

    #[test]
    fn history_extra_can_be_zero_but_kline_extra_cannot() {
        // The two formulas differ in exactly this way, and conflating them would
        // under-count kline traffic.
        assert_eq!(history_extra_weight(19), 0);
        assert_eq!(history_extra_weight(20), 1);
        assert_eq!(history_extra_weight(45), 2);

        assert_eq!(kline_extra_weight(0), 1);
        assert_eq!(kline_extra_weight(24), 1);
        assert_eq!(kline_extra_weight(50), 2);
    }

    #[test]
    fn batch_cost_keeps_the_two_axes_separate() {
        // 100 orders in one request: cheap against the IP budget, expensive against the
        // address limit. Collapsing these into one number is the mistake this type prevents.
        let cost = BatchCost::for_batch(100);

        assert_eq!(cost.ip_weight, 3);
        assert_eq!(cost.order_count, 100);
    }

    #[test]
    fn budget_refuses_once_the_window_is_full() {
        let mut budget = WeightBudget::with_limit(100, 60_000);

        budget.try_consume(60, 1_000).unwrap();
        budget.try_consume(40, 1_000).unwrap();

        let err = budget.try_consume(1, 1_000).unwrap_err();
        assert_eq!(err.axis, Axis::IpWeight);
        assert_eq!(err.available, 0);
    }

    #[test]
    fn budget_recovers_as_the_window_slides() {
        let mut budget = WeightBudget::with_limit(100, 60_000);

        budget.try_consume(100, 1_000).unwrap();
        assert!(budget.try_consume(1, 30_000).is_err(), "still inside window");

        // The entry at t=1000 leaves the window once now - 60_000 reaches it.
        assert_eq!(budget.available(61_001), 100);
        budget.try_consume(100, 61_001).unwrap();
    }

    #[test]
    fn rejection_reports_a_usable_retry_delay() {
        let mut budget = WeightBudget::with_limit(100, 60_000);
        budget.try_consume(100, 5_000).unwrap();

        let err = budget.try_consume(10, 20_000).unwrap_err();

        // Oldest entry sits at t=5000, so it clears at t=65000: 45s from now.
        assert_eq!(err.retry_after_ms, 45_000);
    }

    #[test]
    fn after_the_fact_charges_are_booked_even_past_the_limit() {
        // A history response can push the window over its cap; the request already
        // happened, so the charge is recorded rather than refused.
        let mut budget = WeightBudget::with_limit(100, 60_000);
        budget.try_consume(95, 1_000).unwrap();

        budget.record(history_extra_weight(200), 1_000);

        assert_eq!(budget.available(1_000), 0);
        assert!(budget.try_consume(1, 1_000).is_err());
    }
}
