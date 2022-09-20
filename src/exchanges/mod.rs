pub use binance::BinanceStream;
pub use bitstamp::BitstampStream;

pub mod binance;
pub mod bitstamp;

use async_trait::async_trait;
use futures::SinkExt;
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use tungstenite::Message;

use crate::{currencies::CurrencyPair, Result};

type WebSocket = WebSocketStream<MaybeTlsStream<TcpStream>>;

#[async_trait]
pub trait ExchangeStream
where
    Self: Sized,
{
    type SubscribeMessage: serde::Serialize + Send;

    async fn initialize(currency_pair: &CurrencyPair) -> Result<Self> {
        let mut websocket = Self::connect(currency_pair).await?;

        let message = Self::subscribe_message(currency_pair).await;
        let message = serde_json::to_string(&message).unwrap();
        let message = Message::Text(message);

        if let err @ Err(_) = websocket.send(message).await {
            // If possible, try closing the websocket before returning error.
            let _ = websocket.close(None);
            err?;
        }

        Self::finish_build(websocket).await
    }

    async fn connect(currency_pair: &CurrencyPair) -> Result<WebSocket>;

    async fn subscribe_message(currency_pair: &CurrencyPair) -> Self::SubscribeMessage;

    async fn finish_build(websocket: WebSocket) -> Result<Self>;
}
