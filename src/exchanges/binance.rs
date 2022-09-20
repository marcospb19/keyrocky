use async_trait::async_trait;
use derive_deref::{Deref, DerefMut};
use serde::Serialize;
use tokio_tungstenite::connect_async;

use crate::{
    currencies::CurrencyPair,
    exchanges::{ExchangeStream, WebSocket},
    Result,
};

const BINANCE_WEBSOCKET_BASE_URL: &str = "wss://stream.binance.com:9443/ws";

#[derive(Deref, DerefMut)]
pub struct BinanceStream(WebSocket);

#[async_trait]
impl ExchangeStream for BinanceStream {
    type SubscribeMessage = BinanceSubscribeMessage;

    async fn connect(currency_pair: &CurrencyPair) -> Result<WebSocket> {
        let suffix = currency_pair.as_str();
        let url = format!("{BINANCE_WEBSOCKET_BASE_URL}/{suffix}");
        let (stream, _) = connect_async(url).await?;
        Ok(stream)
    }

    async fn subscribe_message(currency_pair: &CurrencyPair) -> Self::SubscribeMessage {
        BinanceSubscribeMessage::new(currency_pair)
    }

    async fn finish_build(websocket: WebSocket) -> Result<Self> {
        Ok(Self(websocket))
    }
}

#[derive(Serialize)]
pub struct BinanceSubscribeMessage {
    method: String,
    params: Vec<String>,
    id: usize,
}

impl BinanceSubscribeMessage {
    pub fn new(currency_pair: &CurrencyPair) -> Self {
        let symbol = currency_pair.as_str().to_lowercase();
        Self {
            method: "SUBSCRIBE".into(),
            params: vec![format!("{symbol}@depth10@100ms")],
            id: 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_binance_serializing_subscribe_message() {
        let currency_pair: CurrencyPair = "ETHBTC".parse().unwrap();
        let message = BinanceSubscribeMessage::new(&currency_pair);

        let expected = json!({
            "method": "SUBSCRIBE",
            "params": [
                "ethbtc@depth10@100ms"
            ],
            "id": 1
        });

        let result = serde_json::to_value(message).unwrap();

        assert_eq!(result, expected);
    }
}
