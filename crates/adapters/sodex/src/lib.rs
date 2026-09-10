// -------------------------------------------------------------------------------------------------
//  SoDEX integration adapter.
// -------------------------------------------------------------------------------------------------

//! [SoDEX](https://sodex.com) integration adapter.
//!
//! SoDEX is an on-chain orderbook DEX on ValueChain. Despite settling on-chain,
//! trading does not require broadcasting transactions: orders are submitted over
//! plain REST and authenticated with an offline EIP-712 signature, which makes the
//! integration shape closer to a centralized venue than to an AMM.
//!
//! # Credential model
//!
//! Two distinct keys, with deliberately different exposure:
//!
//! - The **master wallet** owns the account. It signs only account-level actions
//!   (`addAPIKey`, `revokeAPIKey`, `approveBuilderFee`) and is expected to stay offline.
//! - An **API key** is a named, revocable signing credential registered by the master
//!   wallet. It signs day-to-day trading actions and cannot read account data. This is
//!   the only key a running process needs to hold.
//!
//! # Signature layout
//!
//! Every signature carries a leading type byte, and the two families differ:
//!
//! - trading actions sign [`ExchangeAction`] under the `spot`/`futures` domain, prefix `0x01`
//! - account-level actions sign under the `universal` domain, prefix `0x02`
//!
//! The payload hash bound into [`ExchangeAction`] is `keccak256` over the *compact* JSON
//! encoding of `{type, params}`. The gateway verifies by parsing the request body into its
//! own Go structs and re-marshaling, so field order is part of the contract — see
//! [`signing`] for how that is preserved on this side.

#![allow(clippy::module_name_repetitions)]

pub mod common;
pub mod config;
pub mod http;
pub mod providers;
pub mod signing;
pub mod websocket;
