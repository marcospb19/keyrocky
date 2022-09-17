use std::io::Write;

use futures::SinkExt;
use serde::Serialize;
use tokio_tungstenite::connect_async;
use tungstenite::Message;

use crate::{currencies::CurrencyPair, exchanges::ExchangeStream, Result};

const BINANCE_WEBSOCKET_BASE_URL: &str = "wss://stream.binance.com:9443/ws";

pub async fn connect_and_subscribe(currency_pair: &CurrencyPair) -> Result<ExchangeStream> {
    let suffix = currency_pair.as_str();
    let url = format!("{BINANCE_WEBSOCKET_BASE_URL}/{suffix}");
    let (mut stream, _) = connect_async(url).await?;

    println!("binance handshake success!!!");
    let _ = std::io::stdout().flush();

    let message = SubscribeMessage::new(currency_pair);
    let message = serde_json::to_string(&message).unwrap();
    let message = Message::Text(message);

    println!("Subscring");

    if let err @ Err(_) = stream.send(message).await {
        // If possible, try closing the stream before returning error.
        let _ = stream.close(None);
        err?;
    }

    println!("Subscribed successfully");

    Ok(stream)
}

#[derive(Serialize)]
struct SubscribeMessage {
    method: String,
    params: Vec<String>,
    id: usize,
}

impl SubscribeMessage {
    pub fn new(currency_pair: &CurrencyPair) -> Self {
        let symbol = currency_pair.as_str().to_lowercase();
        Self {
            method: "SUBSCRIBE".into(),
            params: vec![format!("{symbol}@aggTrade"), format!("{symbol}@depth")],
            id: 1,
        }
    }
}

#[cfg(test)]
mod tests {
    // use serde_json::json;

    // use super::*;

    // #[test]
    // fn test_binance_serializing_subscribe_message() {
    //     let currency_pair: CurrencyPair = "ETHBTC".parse().unwrap();
    //     let message = SubscribeMessage::new(&currency_pair);

    //     let expected = json!({
    //         "event": "bts:subscribe",
    //         "data": {
    //             "channel": "live_orders_ethbtc"
    //         }
    //     });

    //     let result = serde_json::to_value(message).unwrap();

    //     assert_eq!(result, expected);
    // }
}
