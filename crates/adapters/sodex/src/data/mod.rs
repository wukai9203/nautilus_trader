//! Market data.
//!
//! Conversion from the venue's wire types into Nautilus data lives in [`parse`], separated
//! from [`client`] so it can be exercised without a socket.

pub mod client;
pub mod history;
pub mod parse;

pub use client::SodexDataClient;
pub use history::{BarRequest, HistoryError, RpcKline, drop_forming_tail, fetch_bars, max_limit};
pub use parse::{BarMappingError, bar_type_for, interval_to_spec, parse_bar, spec_to_interval};
