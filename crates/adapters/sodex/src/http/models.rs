//! Wire types shared by every REST response.

use serde::{Deserialize, Serialize};

/// The envelope every REST endpoint wraps its payload in.
///
/// Per the venue's REST overview the envelope carries `code` for status, `timestamp` for
/// server time in milliseconds, `error` when the request failed, and `data` for the
/// endpoint-specific payload.
///
/// # On deciding success
///
/// The documentation states that `error` is present when a request fails but does not
/// publish the value `code` takes on success, so [`ApiResponse::into_result`] keys off the
/// presence of `error` rather than asserting a magic number. Once a live round-trip
/// establishes the success code, this can tighten into a check on both fields — until then,
/// inventing a constant would be a guess dressed as a contract.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    /// Venue status code. Retained verbatim for diagnostics.
    pub code: Option<serde_json::Value>,
    /// Server time in milliseconds.
    ///
    /// Worth surfacing rather than discarding: nonces must sit inside a window measured
    /// against venue time, so this is the authoritative reference for detecting clock skew.
    pub timestamp: Option<u64>,
    /// Present when the request failed.
    pub error: Option<String>,
    /// Endpoint-specific payload.
    pub data: Option<T>,
}

impl<T> ApiResponse<T> {
    /// Unwraps the payload, or reports the venue's error.
    ///
    /// # Errors
    ///
    /// Returns [`EnvelopeError::Venue`] when `error` is set, and
    /// [`EnvelopeError::MissingData`] when the request succeeded but carried no payload.
    pub fn into_result(self) -> Result<T, EnvelopeError> {
        if let Some(message) = self.error {
            return Err(EnvelopeError::Venue {
                code: self.code.map(|c| c.to_string()),
                message,
            });
        }
        self.data.ok_or(EnvelopeError::MissingData)
    }
}

/// Failures expressed by the response envelope itself, as opposed to transport failures.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum EnvelopeError {
    #[error("venue rejected the request{}: {message}", .code.as_deref().map_or(String::new(), |c| format!(" (code {c})")))]
    Venue {
        code: Option<String>,
        message: String,
    },
    #[error("response reported success but carried no data payload")]
    MissingData,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct Payload {
        aid: u64,
    }

    #[test]
    fn success_envelope_yields_payload() {
        let raw = r#"{"code":0,"timestamp":1760373925000,"data":{"aid":12345}}"#;
        let parsed: ApiResponse<Payload> = serde_json::from_str(raw).unwrap();

        assert_eq!(parsed.timestamp, Some(1_760_373_925_000));
        assert_eq!(parsed.into_result().unwrap(), Payload { aid: 12345 });
    }

    #[test]
    fn error_envelope_surfaces_message_and_code() {
        let raw = r#"{"code":21104,"timestamp":1760373925000,"error":"invalid nonce"}"#;
        let parsed: ApiResponse<Payload> = serde_json::from_str(raw).unwrap();

        let err = parsed.into_result().unwrap_err();
        assert!(matches!(err, EnvelopeError::Venue { .. }));
        assert!(err.to_string().contains("invalid nonce"), "{err}");
        assert!(err.to_string().contains("21104"), "{err}");
    }

    #[test]
    fn error_takes_precedence_even_when_data_is_present() {
        // Defensive: if the venue ever returns both, the failure must win rather than a
        // partially-populated payload being handed to a caller as success.
        let raw = r#"{"code":1,"error":"rejected","data":{"aid":1}}"#;
        let parsed: ApiResponse<Payload> = serde_json::from_str(raw).unwrap();

        assert!(matches!(
            parsed.into_result(),
            Err(EnvelopeError::Venue { .. })
        ));
    }

    #[test]
    fn success_without_payload_is_distinguishable_from_venue_error() {
        // Some endpoints document "no endpoint-specific data"; callers expecting a payload
        // should see a distinct error rather than a confusing venue message.
        let raw = r#"{"code":0,"timestamp":1760373925000}"#;
        let parsed: ApiResponse<Payload> = serde_json::from_str(raw).unwrap();

        assert_eq!(parsed.into_result().unwrap_err(), EnvelopeError::MissingData);
    }

    #[test]
    fn unknown_envelope_fields_do_not_break_parsing() {
        // The venue can add fields; a strict decoder would turn that into an outage.
        let raw = r#"{"code":0,"timestamp":1,"data":{"aid":7},"newField":"ignored"}"#;
        let parsed: ApiResponse<Payload> = serde_json::from_str(raw).unwrap();

        assert_eq!(parsed.into_result().unwrap(), Payload { aid: 7 });
    }
}
