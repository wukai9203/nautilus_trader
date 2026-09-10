//! Market data.
//!
//! Conversion from the venue's wire types into Nautilus data lives in [`parse`]. The client
//! that drives the WebSocket connection and publishes those events is not implemented yet;
//! the conversion layer is separated so it can be exercised without a socket.

pub mod history;
pub mod parse;

pub use history::{BarRequest, HistoryError, RpcKline, drop_forming_tail, fetch_bars, max_limit};
pub use parse::{BarMappingError, bar_type_for, interval_to_spec, parse_bar, spec_to_interval};
