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

//! # What is implemented
//!
//! | Area | State |
//! |------|-------|
//! | Instruments | Loaded from the venue listing for both engines |
//! | Historical bars | REST klines, with the still-forming tail removed |
//! | Streaming bars | Candle channel, closed bars only |
//! | Order submission | Market and limit, spot and perps |
//! | Order cancellation | By venue order id, falling back to the client order id |
//! | Account state | **Not implemented** |
//! | Order status and fill reports | **Not implemented** |
//! | Position reports | **Not implemented** |
//!
//! The three gaps share one cause: the venue documentation this adapter was built from
//! covers the trading endpoints, and the paths for account, order-status and position reads
//! have not been verified against a live link. Guessing them would produce failures that
//! read like credential errors rather than missing endpoints — a diagnosis this integration
//! has already cost time on once.
//!
//! The consequence is worth stating rather than leaving to be discovered: **the execution
//! client cannot reconcile.** Orders it did not place, and fills that occurred while it was
//! disconnected, remain invisible to the engine. Fills are also not reported at all, so a
//! live run currently learns that an order was accepted but never that it was filled. The
//! venue does publish a user stream that would carry them, and it requires no authorization,
//! but its channel names and payload shapes are not documented here — that stream is the
//! next thing to add, and it should be built against a live link rather than from
//! assumption.
//!
//! # Backtesting
//!
//! Historical bars come back through the same conversion the stream uses, so a backtest and
//! a live run see bars built by identical code. The one asymmetry that would otherwise
//! remain — the venue marks streamed bars closed but leaves historical ones unmarked — is
//! removed on both paths: the stream filters on the venue's flag, and history drops its
//! trailing bar by comparing the bar's open plus its interval against the clock.

#![allow(clippy::module_name_repetitions)]

pub mod common;
pub mod config;
pub mod data;
pub mod execution;
pub mod factories;
pub mod http;
pub mod providers;
pub mod signing;
pub mod websocket;
