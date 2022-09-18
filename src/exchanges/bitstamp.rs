use futures::SinkExt;
use serde::Serialize;
use tokio_tungstenite::connect_async;
use tungstenite::Message;

use crate::{currencies::CurrencyPair, exchanges::ExchangeStream, Result};

const BITSTAMP_WEBSOCKET_URL: &str = "wss://ws.bitstamp.net";

pub async fn connect_and_subscribe(currency_pair: &CurrencyPair) -> Result<ExchangeStream> {
    let (mut stream, _) = connect_async(BITSTAMP_WEBSOCKET_URL).await?;

    let message = SubscribeMessage::new(currency_pair);
    let message = serde_json::to_string(&message).unwrap();
    let message = Message::Text(message);

    if let err @ Err(_) = stream.send(message).await {
        // If possible, try closing the stream before returning error.
        let _ = stream.close(None);
        err?;
    }

    Ok(stream)
}

#[derive(Serialize)]
struct SubscribeMessage {
    event: String,
    data: ChannelInformation,
}

impl SubscribeMessage {
    pub fn new(currency_pair: &CurrencyPair) -> Self {
        Self {
            event: "bts:subscribe".into(),
            data: ChannelInformation::new(currency_pair),
        }
    }
}

#[derive(Serialize)]
struct ChannelInformation {
    channel: String,
}

impl ChannelInformation {
    pub fn new(currency_pair: &CurrencyPair) -> Self {
        let symbol = currency_pair.as_str().to_lowercase();
        Self {
            channel: format!("detail_order_book_{symbol}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_bitstamp_serializing_subscribe_message() {
        let currency_pair: CurrencyPair = "ETHBTC".parse().unwrap();
        let message = SubscribeMessage::new(&currency_pair);

        // Example from https://www.bitstamp.net/websocket/v2/
        let expected = json!({
            "event": "bts:subscribe",
            "data": {
                "channel": "detail_order_book_ethbtc"
            }
        });

        let result = serde_json::to_value(message).unwrap();

        assert_eq!(result, expected);
    }
}
