//! Normalisation of the venue's decimal strings.
//!
//! The venue emits amounts at on-chain token precision — a volume comes back as
//! `"0.001390000000000000"`, eighteen decimal places. Nautilus's fixed-point types cap
//! precision at [`FIXED_PRECISION`] and reject anything longer, so every decimal string
//! crossing this boundary is normalised first.
//!
//! Two steps, in order, because they have different costs:
//!
//! 1. **Strip trailing zeros.** Lossless, and enough on its own for the common case: those
//!    eighteen places are almost always padding.
//! 2. **Round what remains.** Only reached when a value carries more than
//!    [`FIXED_PRECISION`] *significant* decimals, which no tick or step size on this venue
//!    does — the finest observed is `0.00001`.
//!
//! Doing this at the boundary rather than at each call site is what keeps the rest of the
//! adapter free of precision defence.

use std::str::FromStr;

use nautilus_model::types::fixed::FIXED_PRECISION;
use rust_decimal::Decimal;

/// Why a decimal string could not be normalised.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("not a decimal number: {0:?}")]
pub struct DecimalError(pub String);

/// Rewrites a venue decimal string so its precision fits the engine's fixed-point types.
///
/// # Errors
///
/// Returns [`DecimalError`] if the input is not a decimal number.
pub fn normalize(raw: &str) -> Result<String, DecimalError> {
    let value = Decimal::from_str(raw).map_err(|_| DecimalError(raw.to_string()))?;

    // `normalize` drops trailing zeros; most venue values need nothing further.
    let trimmed = value.normalize();

    let capped = if trimmed.scale() > u32::from(FIXED_PRECISION) {
        trimmed.round_dp(u32::from(FIXED_PRECISION))
    } else {
        trimmed
    };

    Ok(capped.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_on_chain_padding_without_changing_the_value() {
        // The case that broke a live fetch: an 18-decimal volume that is really 5 decimals.
        assert_eq!(normalize("0.001390000000000000").unwrap(), "0.00139");
        assert_eq!(normalize("4.12298000").unwrap(), "4.12298");
    }

    #[test]
    fn leaves_already_short_values_alone() {
        assert_eq!(normalize("91976").unwrap(), "91976");
        assert_eq!(normalize("0.00001").unwrap(), "0.00001");
        assert_eq!(normalize("1").unwrap(), "1");
    }

    #[test]
    fn result_never_exceeds_the_engine_precision() {
        // The property that matters: whatever comes in, the output can be parsed by the
        // engine's fixed-point types.
        for raw in [
            "0.001390000000000000",
            "0.123456789012345678",
            "1234.000000000000000001",
            "0.000000000000000001",
        ] {
            let normalized = normalize(raw).unwrap();
            let scale = Decimal::from_str(&normalized).unwrap().scale();
            assert!(
                scale <= u32::from(FIXED_PRECISION),
                "{raw} normalised to {normalized} with scale {scale}"
            );
        }
    }

    #[test]
    fn rounds_only_when_significant_digits_actually_exceed_the_cap() {
        // Rounding is lossy, so it must not trigger on padding alone. This value has more
        // significant decimals than the cap and is the one case that legitimately rounds.
        let rounded = normalize("0.123456789012345678").unwrap();
        let scale = Decimal::from_str(&rounded).unwrap().scale();

        assert_eq!(scale, u32::from(FIXED_PRECISION));
        assert!(rounded.starts_with("0.1234567"));
    }

    #[test]
    fn a_value_below_the_representable_minimum_becomes_zero_not_an_error() {
        // 1e-18 cannot be represented; rounding yields zero. Surfacing that as a parse
        // failure would drop an entire series over one dust-sized field.
        let normalized = normalize("0.000000000000000001").unwrap();

        assert_eq!(Decimal::from_str(&normalized).unwrap(), Decimal::ZERO);
    }

    #[test]
    fn negative_values_survive() {
        // Funding rates and PnL can be negative; normalisation must not mangle the sign.
        assert_eq!(normalize("-0.0000125000").unwrap(), "-0.0000125");
    }

    #[test]
    fn non_numeric_input_is_reported_rather_than_defaulted() {
        assert!(normalize("").is_err());
        assert!(normalize("abc").is_err());
        assert!(normalize("1.2.3").is_err());
    }
}
