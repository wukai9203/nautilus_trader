//! EIP-712 signing for SoDEX.
//!
//! # Why field order is a correctness concern here
//!
//! The signed digest is not over the typed struct alone. It is over
//! `keccak256(compact_json({type, params}))`, and the gateway verifies by parsing the
//! request body into its own Go structs and re-marshaling with `json.Marshal`, which
//! emits fields in struct-definition order. A payload whose keys are ordered differently
//! hashes differently and the signature is rejected — with no diagnostic beyond a
//! verification failure.
//!
//! `serde` serializes struct fields in declaration order, which matches Go's behaviour, so
//! the contract holds as long as payloads are modeled as concrete structs whose field order
//! mirrors the Go SDK. It does **not** hold for [`serde_json::Value`], whose object
//! representation is a `BTreeMap` and therefore re-sorts keys alphabetically. For that
//! reason [`payload_hash`] is generic over `T: Serialize` rather than taking a `Value`.
//!
//! Three further encoding rules travel with the order requirement:
//!
//! - fields typed `DecimalString` are quoted strings (`"quantity":"0.001"`), never numbers
//! - `omitempty` fields must be absent entirely when unset, i.e. `Option` +
//!   `skip_serializing_if`
//! - non-optional fields must be present even at their zero value

pub mod nonce;
pub mod signers;
pub mod universal;

pub use nonce::{NonceGenerator, is_within_window};
pub use signers::{ExchangeSigner, SigningError, payload_hash};
pub use universal::UniversalSigner;
