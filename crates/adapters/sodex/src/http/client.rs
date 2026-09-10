//! REST client.
//!
//! # Signing and the request body differ on purpose
//!
//! The signature commits to `keccak256` over `{"type": …, "params": …}`, but the HTTP body
//! carries **only the `params` object** — the venue documents this explicitly:
//!
//! > the HTTP request body contains only the params object (without the type wrapper),
//! > using the same field order and types as the signing payload.
//!
//! Sending the wrapped form, or hashing the unwrapped one, both fail verification with the
//! same opaque message. [`SignedRequest`] is produced without touching the network so this
//! step is testable on its own rather than only observable as a rejection.

use std::collections::HashMap;

use nautilus_network::http::{HttpClient, HttpClientError, HttpResponse, Method};
use serde::Serialize;

use super::{
    Network,
    models::{ApiResponse, EnvelopeError},
    ratelimit::{RateLimited, WeightBudget},
    requests::RequestError,
};
use crate::{
    common::{
        Market,
        credential::{ApiKeyName, ApiPrivateKey, CredentialError},
    },
    signing::{
        NonceGenerator,
        signers::{ExchangeSigner, SigningError, payload_hash},
    },
};

/// Header carrying the API key's *name*.
pub const HEADER_API_KEY: &str = "X-API-Key";
/// Header carrying the typed signature.
pub const HEADER_API_SIGN: &str = "X-API-Sign";
/// Header carrying the nonce.
pub const HEADER_API_NONCE: &str = "X-API-Nonce";

/// Failures from the REST layer.
#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("credentials required for signed requests")]
    CredentialsRequired,
    #[error(transparent)]
    Credential(#[from] CredentialError),
    #[error(transparent)]
    Signing(#[from] SigningError),
    #[error(transparent)]
    Request(#[from] RequestError),
    #[error(transparent)]
    RateLimited(#[from] RateLimited),
    #[error("serialization failed: {0}")]
    Serialize(#[from] serde_json::Error),
    #[error("transport failed: {0}")]
    Transport(String),
    #[error("venue returned HTTP {status}: {body}")]
    Status { status: u16, body: String },
}

impl From<HttpClientError> for ClientError {
    fn from(error: HttpClientError) -> Self {
        Self::Transport(error.to_string())
    }
}

/// A fully prepared signed request, before it touches the network.
///
/// Separated from sending so the signing and encoding rules can be asserted directly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignedRequest {
    pub method: Method,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl SignedRequest {
    /// The body as text, for assertions and diagnostics.
    #[must_use]
    pub fn body_str(&self) -> String {
        String::from_utf8_lossy(&self.body).into_owned()
    }
}

/// Credentials for signed endpoints.
#[derive(Debug)]
struct Credentials {
    key_name: ApiKeyName,
    signer: ExchangeSigner,
    nonces: NonceGenerator,
}

/// REST client for one network and market.
///
/// Bound to a single market because the EIP-712 domain differs between spot and perps; a
/// client that could switch would be able to sign a perps action under the spot domain.
#[derive(Debug)]
pub struct SodexHttpClient {
    http: HttpClient,
    network: Network,
    market: Market,
    credentials: Option<Credentials>,
    weights: std::sync::Mutex<WeightBudget>,
}

impl SodexHttpClient {
    /// A client for unsigned market-data endpoints.
    ///
    /// # Errors
    ///
    /// Returns [`ClientError::Transport`] if the underlying HTTP client cannot be built.
    pub fn new_public(network: Network, market: Market) -> Result<Self, ClientError> {
        Ok(Self {
            http: Self::build_http()?,
            network,
            market,
            credentials: None,
            weights: std::sync::Mutex::new(WeightBudget::new()),
        })
    }

    /// A client that can sign trading actions.
    ///
    /// Takes an [`ApiPrivateKey`] rather than the master key: trading actions must be signed
    /// by a registered API key, and the master wallet is expected to stay offline.
    ///
    /// # Errors
    ///
    /// Returns [`ClientError`] if the key cannot be parsed into a signer or the HTTP client
    /// cannot be built.
    pub fn with_credentials(
        network: Network,
        market: Market,
        key_name: ApiKeyName,
        key: &ApiPrivateKey,
    ) -> Result<Self, ClientError> {
        let signer = ExchangeSigner::new(key, market, network.chain_id())?;
        Ok(Self {
            http: Self::build_http()?,
            network,
            market,
            credentials: Some(Credentials {
                key_name,
                signer,
                nonces: NonceGenerator::new(),
            }),
            weights: std::sync::Mutex::new(WeightBudget::new()),
        })
    }

    /// Whether this client can sign.
    #[must_use]
    pub const fn can_sign(&self) -> bool {
        self.credentials.is_some()
    }

    /// Absolute URL for a market-scoped path such as `/trade/orders`.
    #[must_use]
    pub fn url_for(&self, path: &str) -> String {
        format!("{}{path}", self.network.market_base(self.market))
    }

    /// Prepares a signed request without sending it.
    ///
    /// `action_type` is the venue's action name (`newOrder`, `cancelOrder`, …) that goes into
    /// the signing payload but **not** into the body.
    ///
    /// # Errors
    ///
    /// Returns [`ClientError::CredentialsRequired`] without credentials, or a signing or
    /// serialization failure.
    pub fn build_signed<P: Serialize>(
        &self,
        method: Method,
        path: &str,
        action_type: &str,
        params: &P,
    ) -> Result<SignedRequest, ClientError> {
        let credentials = self
            .credentials
            .as_ref()
            .ok_or(ClientError::CredentialsRequired)?;

        let digest = payload_hash(action_type, params)?;
        let nonce = credentials.nonces.next();
        let signature = credentials.signer.sign_action(digest, nonce)?;

        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers.insert("Accept".to_string(), "application/json".to_string());
        headers.insert(
            HEADER_API_KEY.to_string(),
            credentials.key_name.as_str().to_string(),
        );
        headers.insert(
            HEADER_API_SIGN.to_string(),
            alloy_primitives::hex::encode_prefixed(&signature),
        );
        headers.insert(HEADER_API_NONCE.to_string(), nonce.to_string());

        // Only `params` goes on the wire; the `{type, params}` envelope exists solely to be
        // hashed. Serializing the envelope here would break verification.
        let body = serde_json::to_vec(params)?;

        Ok(SignedRequest {
            method,
            url: self.url_for(path),
            headers,
            body,
        })
    }

    /// Reserves request weight against the rolling per-IP budget.
    ///
    /// # Errors
    ///
    /// Returns [`ClientError::RateLimited`] carrying how long to wait.
    pub fn reserve_weight(&self, weight: u32, now_ms: u64) -> Result<(), ClientError> {
        self.weights
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .try_consume(weight, now_ms)
            .map_err(ClientError::from)
    }

    /// Books weight the venue charges after a response, such as history row counts.
    pub fn record_weight(&self, weight: u32, now_ms: u64) {
        self.weights
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .record(weight, now_ms);
    }

    /// Sends a prepared request and decodes the envelope.
    ///
    /// # Errors
    ///
    /// Returns [`ClientError::Status`] for a non-success HTTP status, or a transport,
    /// decoding or venue-level error.
    pub async fn send<T: serde::de::DeserializeOwned>(
        &self,
        request: SignedRequest,
    ) -> Result<T, ClientError> {
        let response = self
            .http
            .request(
                request.method,
                request.url,
                None,
                Some(request.headers),
                Some(request.body),
                None,
                None,
            )
            .await?;

        Self::decode(response)
    }

    /// Sends a prepared request against an endpoint that may return no payload.
    ///
    /// Several trading endpoints — `scheduleCancel`, `updateLeverage`, `updateMargin`,
    /// `modifyOrder` — document "no endpoint-specific data". For those, an absent `data` is
    /// the success case, not the missing-payload error [`send`](Self::send) reports.
    ///
    /// # Errors
    ///
    /// Returns [`ClientError`] on transport, status or venue-level failure. An accepted
    /// request that returns nothing yields `Ok(None)`.
    pub async fn send_optional<T: serde::de::DeserializeOwned>(
        &self,
        request: SignedRequest,
    ) -> Result<Option<T>, ClientError> {
        let response = self
            .http
            .request(
                request.method,
                request.url,
                None,
                Some(request.headers),
                Some(request.body),
                None,
                None,
            )
            .await?;

        let status = response.status.as_u16();
        if !(200..300).contains(&status) {
            return Err(ClientError::Status {
                status,
                body: String::from_utf8_lossy(&response.body).into_owned(),
            });
        }

        let envelope: ApiResponse<T> = serde_json::from_slice(&response.body)?;
        match envelope.into_result() {
            Ok(data) => Ok(Some(data)),
            Err(EnvelopeError::MissingData) => Ok(None),
            Err(other) => Err(ClientError::Transport(other.to_string())),
        }
    }

    /// Sends an unsigned GET against a market-data path.
    ///
    /// # Errors
    ///
    /// Returns [`ClientError`] on transport, status or decoding failure.
    pub async fn get_public<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        params: Option<&HashMap<String, Vec<String>>>,
    ) -> Result<T, ClientError> {
        let mut headers = HashMap::new();
        headers.insert("Accept".to_string(), "application/json".to_string());

        let response = self
            .http
            .request(
                Method::GET,
                self.url_for(path),
                params,
                Some(headers),
                None,
                None,
                None,
            )
            .await?;

        Self::decode(response)
    }

    fn decode<T: serde::de::DeserializeOwned>(
        response: HttpResponse,
    ) -> Result<T, ClientError> {
        let status = response.status.as_u16();
        let body = response.body;

        if !(200..300).contains(&status) {
            return Err(ClientError::Status {
                status,
                body: String::from_utf8_lossy(&body).into_owned(),
            });
        }

        let envelope: ApiResponse<T> = serde_json::from_slice(&body)?;
        envelope
            .into_result()
            .map_err(|e| ClientError::Transport(e.to_string()))
    }

    fn build_http() -> Result<HttpClient, ClientError> {
        HttpClient::builder()
            .headers(HashMap::from([(
                "Accept".to_string(),
                "application/json".to_string(),
            )]))
            .rate_limiters(Vec::new())
            .timeout_secs(30)
            .build()
            .map_err(ClientError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        common::enums::OrderSide,
        http::requests::{ClientOrderId, NewOrderRequest, OrderItem},
    };

    fn client() -> SodexHttpClient {
        let key = ApiPrivateKey::parse(&"2".repeat(64)).unwrap();
        let name = ApiKeyName::parse("api-key-01").unwrap();
        SodexHttpClient::with_credentials(Network::Testnet, Market::Perps, name, &key).unwrap()
    }

    fn order() -> NewOrderRequest {
        NewOrderRequest::new(
            12345,
            1,
            vec![OrderItem::market(
                ClientOrderId::parse("my-order-1").unwrap(),
                OrderSide::Buy,
                "0.001",
            )],
        )
        .unwrap()
    }

    #[test]
    fn body_carries_params_only_without_the_signing_envelope() {
        // The venue hashes {type, params} but expects only params on the wire. Sending the
        // envelope would fail verification with an unhelpful message.
        let request = client()
            .build_signed(Method::POST, "/trade/orders", "newOrder", &order())
            .unwrap();

        let body = request.body_str();

        // Note the assertion is on the envelope's own keys, not on `"type"` alone: an order
        // item legitimately carries a `type` field, so a bare substring check would fail on
        // correct output.
        assert!(!body.contains(r#""type":"newOrder""#), "envelope leaked: {body}");
        assert!(!body.contains(r#""params":"#), "envelope leaked: {body}");
        assert!(body.starts_with(r#"{"accountID":12345"#), "{body}");
    }

    #[test]
    fn body_matches_the_signed_params_byte_for_byte() {
        let request = client()
            .build_signed(Method::POST, "/trade/orders", "newOrder", &order())
            .unwrap();

        // Any divergence between what was hashed and what is sent breaks verification.
        assert_eq!(request.body, serde_json::to_vec(&order()).unwrap());
    }

    #[test]
    fn api_key_header_carries_the_name_not_the_address() {
        // The venue lists this confusion as the most common integration error.
        let request = client()
            .build_signed(Method::POST, "/trade/orders", "newOrder", &order())
            .unwrap();

        assert_eq!(request.headers.get(HEADER_API_KEY).unwrap(), "api-key-01");
        assert!(!request.headers[HEADER_API_KEY].starts_with("0x"));
    }

    #[test]
    fn signature_header_is_hex_with_the_exchange_prefix() {
        let request = client()
            .build_signed(Method::POST, "/trade/orders", "newOrder", &order())
            .unwrap();

        let signature = request.headers.get(HEADER_API_SIGN).unwrap();
        assert!(signature.starts_with("0x01"), "{signature}");
        // 0x + 66 bytes (1 prefix + 65 signature) as hex.
        assert_eq!(signature.len(), 2 + 66 * 2);
    }

    #[test]
    fn every_request_draws_a_fresh_nonce() {
        let client = client();

        let first = client
            .build_signed(Method::POST, "/trade/orders", "newOrder", &order())
            .unwrap();
        let second = client
            .build_signed(Method::POST, "/trade/orders", "newOrder", &order())
            .unwrap();

        let a: u64 = first.headers[HEADER_API_NONCE].parse().unwrap();
        let b: u64 = second.headers[HEADER_API_NONCE].parse().unwrap();
        assert!(b > a, "nonces must advance: {a} then {b}");
    }

    #[test]
    fn identical_payloads_produce_different_signatures_via_the_nonce() {
        // The nonce is inside the signed struct, so replaying a body is not enough.
        let client = client();
        let first = client
            .build_signed(Method::POST, "/trade/orders", "newOrder", &order())
            .unwrap();
        let second = client
            .build_signed(Method::POST, "/trade/orders", "newOrder", &order())
            .unwrap();

        assert_eq!(first.body, second.body);
        assert_ne!(
            first.headers[HEADER_API_SIGN],
            second.headers[HEADER_API_SIGN]
        );
    }

    #[test]
    fn public_client_refuses_to_sign() {
        let public = SodexHttpClient::new_public(Network::Testnet, Market::Perps).unwrap();

        assert!(!public.can_sign());
        assert!(matches!(
            public.build_signed(Method::POST, "/trade/orders", "newOrder", &order()),
            Err(ClientError::CredentialsRequired)
        ));
    }

    #[test]
    fn urls_are_scoped_to_the_configured_network_and_market() {
        assert_eq!(
            client().url_for("/trade/orders"),
            "https://testnet-gw.sodex.dev/api/v1/perps/trade/orders"
        );
    }

    #[test]
    fn weight_budget_is_shared_across_calls() {
        let client = client();

        client.reserve_weight(1100, 1_000).unwrap();
        assert!(matches!(
            client.reserve_weight(200, 1_000),
            Err(ClientError::RateLimited(_))
        ));
    }

    #[test]
    fn after_the_fact_weight_is_recorded_against_the_same_budget() {
        let client = client();

        client.reserve_weight(1000, 1_000).unwrap();
        client.record_weight(200, 1_000);

        assert!(matches!(
            client.reserve_weight(1, 1_000),
            Err(ClientError::RateLimited(_))
        ));
    }
}
