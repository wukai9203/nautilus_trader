//! Order execution.
//!
//! Translation between Nautilus orders and venue requests lives in [`parse`]. The client
//! that drives submission and publishes execution reports is not implemented yet; the
//! conversion layer is separated so it can be exercised without a connection.

pub mod parse;

pub use parse::{
    OrderSpec,
    OrderConversionError, is_fill, map_client_order_id, map_order_status, map_order_type,
    map_side, map_time_in_force, to_perps_order, to_spot_order,
};
