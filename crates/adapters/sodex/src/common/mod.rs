//! Shared types and constants for the SoDEX adapter.

pub mod credential;
pub mod decimal;
pub mod enums;

/// ValueChain mainnet, used as `message.chainID` and as the trading-action EIP-712 `chainId`.
pub const CHAIN_ID_MAINNET: u64 = 286623;

/// ValueChain testnet.
pub const CHAIN_ID_TESTNET: u64 = 138565;

/// Which orderbook an action targets. Also selects the EIP-712 domain name, which is why
/// this is not merely cosmetic: signing a perps action under the `spot` domain produces a
/// signature the gateway will reject.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Market {
    Spot,
    Perps,
}

impl Market {
    /// The EIP-712 `domain.name` for this market.
    ///
    /// Note the asymmetry with the REST path segment: perps actions sign under `futures`,
    /// not `perps`.
    #[must_use]
    pub const fn domain_name(self) -> &'static str {
        match self {
            Self::Spot => "spot",
            Self::Perps => "futures",
        }
    }

    /// The REST path segment for this market (`/api/v1/{segment}`).
    #[must_use]
    pub const fn path_segment(self) -> &'static str {
        match self {
            Self::Spot => "spot",
            Self::Perps => "perps",
        }
    }
}

/// Leading byte identifying the signature family. The gateway rejects raw 65-byte
/// signatures that carry no prefix.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignatureKind {
    /// Trading actions signed under the `spot`/`futures` domain.
    Exchange,
    /// Account-level actions (`addAPIKey`, `revokeAPIKey`, `approveBuilderFee`)
    /// signed under the `universal` domain by the master wallet.
    Universal,
}

impl SignatureKind {
    #[must_use]
    pub const fn prefix(self) -> u8 {
        match self {
            Self::Exchange => 0x01,
            Self::Universal => 0x02,
        }
    }
}
