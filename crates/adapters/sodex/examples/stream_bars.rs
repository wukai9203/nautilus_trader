//! Streams live candles from the venue and reports which ones are safe to trade on.
//!
//! Needs no credentials — market data is served unsigned. The program subscribes, prints
//! every push, and marks whether the bar is closed. Most pushes are the same bar forming;
//! only the ones marked `CLOSED` reach a strategy.
//!
//! It also exercises the parts of the client that unit tests cannot: the connection itself,
//! the venue's acknowledgement shape, whether the keepalive is accepted, and whether the
//! symbol and interval come back spelled the way they were sent.
//!
//! # Usage
//!
//! ```text
//! cargo run -p nautilus-sodex --example stream_bars
//! ```
//!
//! Optional overrides: `SODEX_NETWORK` (default testnet), `SODEX_MARKET` (`spot` or
//! `perps`, default spot), `SODEX_SYMBOL` (default `vBTC_vUSDC`), `SODEX_INTERVAL`
//! (default `1m`), `SODEX_SECONDS` (how long to listen, default 90).

use std::{env, time::Duration};

use nautilus_sodex::{
    common::Market,
    http::Network,
    websocket::{CandleParams, SodexWebSocketClient, SodexWsEvent},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let network = match env::var("SODEX_NETWORK").as_deref() {
        Ok("mainnet") => Network::Mainnet,
        _ => Network::Testnet,
    };
    let market = match env::var("SODEX_MARKET").as_deref() {
        Ok("perps") => Market::Perps,
        _ => Market::Spot,
    };
    let symbol = env::var("SODEX_SYMBOL").unwrap_or_else(|_| "vBTC_vUSDC".to_string());
    let interval = env::var("SODEX_INTERVAL").unwrap_or_else(|_| "1m".to_string());
    let seconds: u64 = env::var("SODEX_SECONDS")
        .unwrap_or_else(|_| "90".into())
        .parse()?;

    let client = SodexWebSocketClient::new(network, market);
    println!("connecting to {network:?} {market:?} …");
    let mut events = client.connect().await?;
    println!("connected");

    client
        .subscribe_candles(CandleParams::new(&symbol, &interval))
        .await?;
    println!("subscribed to {symbol} {interval}; listening for {seconds}s");
    println!();

    let deadline = tokio::time::sleep(Duration::from_secs(seconds));
    tokio::pin!(deadline);

    let mut forming = 0_u32;
    let mut closed = 0_u32;

    loop {
        tokio::select! {
            () = &mut deadline => break,
            event = events.recv() => match event {
                Some(SodexWsEvent::Candle(candle)) => {
                    if candle.is_final() {
                        closed += 1;
                        println!(
                            "CLOSED  {} {} open={} o={} h={} l={} c={} v={}",
                            candle.symbol, candle.interval, candle.open_time_ms,
                            candle.open, candle.high, candle.low, candle.close, candle.volume,
                        );
                    } else {
                        forming += 1;
                        println!(
                            "forming {} {} open={} c={} (bar still open — not tradable)",
                            candle.symbol, candle.interval, candle.open_time_ms, candle.close,
                        );
                    }
                }
                Some(SodexWsEvent::RequestRejected { op, params, reason }) => {
                    println!("REJECTED {op:?} {params:?}: {reason}");
                }
                Some(SodexWsEvent::Reconnected) => println!("-- reconnected, subscriptions replayed --"),
                None => {
                    println!("stream ended");
                    break;
                }
            },
        }
    }

    println!();
    println!("{forming} forming pushes, {closed} closed bars in {seconds}s");
    if closed == 0 {
        println!(
            "No closed bar arrived. With a {interval} interval that is expected unless the run \
             spans a boundary — the forming count above is what proves the feed is live."
        );
    }

    client
        .unsubscribe_candles(&CandleParams::new(&symbol, &interval))
        .await?;
    client.close().await;
    println!("closed");

    Ok(())
}
