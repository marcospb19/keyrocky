use async_trait::async_trait;
use derive_deref::{Deref, DerefMut};
use serde::Serialize;
use tokio_tungstenite::connect_async;

use crate::{
    currencies::CurrencyPair,
    exchanges::{ExchangeStream, WebSocket},
    Result,
};

const BITSTAMP_WEBSOCKET_URL: &str = "wss://ws.bitstamp.net";

#[derive(Deref, DerefMut)]
pub struct BitstampStream(WebSocket);

#[async_trait]
impl ExchangeStream for BitstampStream {
    type SubscribeMessage = BitstampSubscribeMessage;

    async fn connect(_currency_pair: &CurrencyPair) -> Result<WebSocket> {
        let (stream, _) = connect_async(BITSTAMP_WEBSOCKET_URL).await?;
        Ok(stream)
    }

    async fn subscribe_message(currency_pair: &CurrencyPair) -> Self::SubscribeMessage {
        BitstampSubscribeMessage::new(currency_pair)
    }

    async fn finish_build(websocket: WebSocket) -> Result<Self> {
        Ok(Self(websocket))
    }
}

#[derive(Serialize)]
pub struct BitstampSubscribeMessage {
    event: String,
    data: ChannelInformation,
}

impl BitstampSubscribeMessage {
    pub fn new(currency_pair: &CurrencyPair) -> Self {
        Self {
            event: "bts:subscribe".into(),
            data: ChannelInformation::new(currency_pair),
        }
    }
}

#[derive(Serialize)]
pub struct ChannelInformation {
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
        let message = BitstampSubscribeMessage::new(&currency_pair);

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
