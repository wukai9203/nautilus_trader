//! Verifies the trading-domain signing path against the live venue.
//!
//! Sends a `scheduleCancel` with no timestamp, which clears any pending scheduled cancel.
//! If none is pending the call changes nothing, so this exercises signature verification,
//! nonce acceptance and header encoding without placing an order or touching a position.
//!
//! This is the Layer 3 gate: the venue publishes no expected signature bytes, so agreement
//! with the documentation cannot establish that a signature will be accepted. Only a live
//! request can.
//!
//! # Usage
//!
//! ```text
//! export SODEX_API_KEY_NAME=api-key-01
//! export SODEX_API_PRIVATE_KEY=<from register_api_key>
//! export SODEX_ACCOUNT_ID=60366
//! export SODEX_NETWORK=testnet
//! cargo run -p nautilus-sodex --example verify_signing
//! ```

use std::env;

use nautilus_network::http::Method;
use nautilus_sodex::{
    common::{
        Market,
        credential::{ApiKeyName, ApiPrivateKey},
    },
    http::{Network, SodexHttpClient, requests::ScheduleCancelRequest},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let key_hex =
        env::var("SODEX_API_PRIVATE_KEY").map_err(|_| "SODEX_API_PRIVATE_KEY is not set")?;
    let key_name = env::var("SODEX_API_KEY_NAME").unwrap_or_else(|_| "api-key-01".to_string());
    let account_id: u64 = env::var("SODEX_ACCOUNT_ID")
        .map_err(|_| "SODEX_ACCOUNT_ID is not set")?
        .parse()?;
    let network = match env::var("SODEX_NETWORK").as_deref() {
        Ok("mainnet") => Network::Mainnet,
        _ => Network::Testnet,
    };

    let key = ApiPrivateKey::parse(&key_hex)?;
    let name = ApiKeyName::parse(&key_name)?;
    let client = SodexHttpClient::with_credentials(network, Market::Perps, name, &key)?;

    let body = ScheduleCancelRequest::clear(account_id);
    let request = client.build_signed(
        Method::POST,
        "/trade/orders/schedule-cancel",
        "scheduleCancel",
        &body,
    )?;

    println!("network:   {network:?}");
    println!("account:   {account_id}");
    println!("key name:  {key_name}");
    println!("url:       {}", request.url);
    println!("body:      {}", request.body_str());
    println!(
        "signature: {}...",
        &request.headers["X-API-Sign"][..12]
    );
    println!();

    match client.send::<serde_json::Value>(request).await {
        Ok(data) => {
            println!("ACCEPTED — trading-domain signing verified end to end");
            println!("response data: {data:?}");
        }
        Err(error) => {
            // A rejection here is still informative: a signature error means the signing
            // path is wrong, while a business error means the signature was accepted and
            // the request merely had nothing to do.
            println!("REJECTED: {error}");
            println!();
            println!("If the message mentions a signature, signer or recovery id, the signing");
            println!("path is at fault. Anything else means the signature was accepted.");
            return Err(error.into());
        }
    }

    Ok(())
}
