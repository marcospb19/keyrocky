//! Implementation of order book connection for different exchanges.

pub use self::{binance::BinanceExchange, bitstamp::BitstampExchange};

mod binance;
mod bitstamp;

use async_trait::async_trait;
use futures::{SinkExt, StreamExt};
use serde::Serialize;
use tungstenite::Message;

use crate::{
    currencies::CurrencyPair,
    websocket::{websocket_connect, WebSocket},
    Result,
};

/// A trait for connecting to an exchange order book.
///
/// Connecting to an order book consists in two steps:
/// - Connect websocket to specific URL.
/// - Send message to subscribe to a specific order book channel.
///
/// An exchange that implements `connect_url` and `subscribe_message`
/// can call `connect_to_order_book` to receive a ready-to-use websocket.
#[async_trait]
pub trait ConnectToOrderBook {
    type SubscribeMessage: Serialize + Send;

    async fn connect_to_order_book(currency_pair: &CurrencyPair) -> Result<WebSocket> {
        let url = Self::connect_url(currency_pair);

        let mut websocket = websocket_connect(url).await?;

        let subscribe_message = Self::subscribe_message(currency_pair);
        let subscribe_message = serde_json::to_string(&subscribe_message).unwrap();
        let subscribe_message = Message::Text(subscribe_message);

        if let err @ Err(_) = websocket.send(subscribe_message).await {
            // If possible, try closing the websocket before returning error
            let _ = websocket.close(Default::default());
            err?;
        }

        // Skip the subscription response
        if let Some(err @ Err(_)) = websocket.next().await {
            err?;
        }

        Ok(websocket)
    }

    fn connect_url(currency_pair: &CurrencyPair) -> String;

    fn subscribe_message(currency_pair: &CurrencyPair) -> Self::SubscribeMessage;
}
