//! Registers a fresh SoDEX API key using the master wallet.
//!
//! The venue's UI exposes no API key management, so registration goes through the
//! `addAPIKey` endpoint, which the master wallet must sign. This runs that once.
//!
//! # Usage
//!
//! ```text
//! export SODEX_MASTER_PRIVATE_KEY=<exported from Settings → Export Email Wallet>
//! export SODEX_ACCOUNT_ID=<the "aid" from /accounts/{address}/state>
//! export SODEX_API_KEY_NAME=api-key-01
//! export SODEX_NETWORK=testnet          # or mainnet
//! cargo run -p nautilus-sodex --example register_api_key
//! ```
//!
//! The generated API private key is printed once and never written anywhere. Store it before
//! the terminal scrolls away; losing it means revoking the key and registering another.
//!
//! The master key is read from the environment and used only to sign this one action. It is
//! never logged, never sent (only the signature it produces is), and should go back offline
//! afterwards — it can authorize withdrawals, while the key being registered cannot even
//! read account data.

use std::env;

use nautilus_sodex::{
    common::{
        Market,
        credential::{ApiKeyName, MasterPrivateKey},
    },
    http::{
        Network,
        account::{AccountClient, NO_EXPIRY, generate_api_key},
    },
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let master_hex = env::var("SODEX_MASTER_PRIVATE_KEY")
        .map_err(|_| "SODEX_MASTER_PRIVATE_KEY is not set")?;
    let account_id: u64 = env::var("SODEX_ACCOUNT_ID")
        .map_err(|_| "SODEX_ACCOUNT_ID is not set")?
        .parse()?;
    let key_name = env::var("SODEX_API_KEY_NAME").unwrap_or_else(|_| "api-key-01".to_string());
    let network = match env::var("SODEX_NETWORK").as_deref() {
        Ok("mainnet") => Network::Mainnet,
        _ => Network::Testnet,
    };

    let master = MasterPrivateKey::parse(&master_hex)?;
    let name = ApiKeyName::parse(&key_name)?;

    // Perps and spot share one account and one API key set; the market only selects the
    // gateway path this request is sent to.
    let client = AccountClient::new(network, Market::Perps, &master)?;

    println!("network:        {network:?}");
    println!("account id:     {account_id}");
    println!("master address: {:?}", client.master_address());
    println!("key name:       {key_name}");
    println!();

    let generated = generate_api_key()?;
    println!("generated API key address: {:?}", generated.public_key);

    let request = client.build_add_api_key(
        account_id,
        &name,
        generated.public_key,
        NO_EXPIRY,
        None, // all permissions enabled; pass a DisabledPermissions mask to restrict
    )?;

    println!("submitting to {} ...", request.url);
    let response: Option<serde_json::Value> = client.send(request).await?;
    println!("venue accepted the registration: {response:?}");

    println!();
    println!("=== store this now; it is not recoverable ===");
    println!("SODEX_API_KEY_NAME={key_name}");
    println!("SODEX_API_PRIVATE_KEY={}", generated.private_key.as_hex());
    println!("=== the master key can go back offline ===");

    Ok(())
}
