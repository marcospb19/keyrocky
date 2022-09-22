use std::{
    pin::Pin,
    str::FromStr,
    task::{ready, Context, Poll},
};

use bigdecimal::BigDecimal;
use futures::{SinkExt, Stream, StreamExt};
use serde::{Deserialize, Serialize};
use tokio_tungstenite::connect_async;
use tungstenite::Message;

use crate::{
    currencies::CurrencyPair,
    exchanges::{Order, OrderBook, WebSocket},
    Error, Result,
};

const BINANCE_WEBSOCKET_BASE_URL: &str = "wss://stream.binance.com:9443/ws";

pub struct BinanceStream(WebSocket);

impl BinanceStream {
    pub async fn connect_and_subscribe(currency_pair: &CurrencyPair) -> Result<Self> {
        let suffix = currency_pair.as_str();
        let url = format!("{BINANCE_WEBSOCKET_BASE_URL}/{suffix}");
        let (mut websocket, _) = connect_async(url).await?;

        let message = BinanceSubscribeMessage::new(currency_pair);
        let message = serde_json::to_string(&message).unwrap();
        let message = Message::Text(message);

        if let err @ Err(_) = websocket.send(message).await {
            // If possible, try closing the websocket before returning error.
            let _ = websocket.close(Default::default());
            err?;
        }

        // Skip the subscription response.
        if let Some(err @ Err(_)) = websocket.next().await {
            err?;
        }

        Ok(Self(websocket))
    }

    fn try_message_to_order_book(message: Message) -> Result<OrderBook> {
        let message_text = message.into_text()?;
        let BinanceRawOrderBook { mut bids, mut asks } =
            serde_json::from_str(&message_text).unwrap();

        if asks.len() < 10 {
            return Err(Error::NotEnoughOrders("Binance".into(), "bids".into()));
        }

        if asks.len() < 10 {
            return Err(Error::NotEnoughOrders("Binance".into(), "asks".into()));
        }

        asks.resize_with(10, || unreachable!());
        bids.resize_with(10, || unreachable!());

        let array_into_order = |array: RawOrder| -> Result<Order, <BigDecimal as FromStr>::Err> {
            let [price, quantity] = array;

            let price = price.parse()?;
            let quantity = quantity.parse()?;

            Ok(Order { price, quantity })
        };

        let bids = bids
            .into_iter()
            .map(array_into_order)
            .collect::<Result<_, _>>()?;

        let asks = asks
            .into_iter()
            .map(array_into_order)
            .collect::<Result<_, _>>()?;

        Ok(OrderBook::new(bids, asks, "Binance"))
    }
}

impl Stream for BinanceStream {
    type Item = Result<OrderBook>;

    fn poll_next(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let message = ready!(Pin::new(&mut self.0).poll_next(context)?);
        message.map(Self::try_message_to_order_book).into()
    }
}

type RawOrder = [String; 2];

#[derive(Deserialize)]
struct BinanceRawOrderBook {
    bids: Vec<RawOrder>,
    asks: Vec<RawOrder>,
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
    fn test_binance_deserializing_order_book() {
        let raw_json = include_str!("../../test_data/binance_order_book_update_message.json");

        let convert_matrix_to_order_list = |matrix: &[[&str; 2]]| {
            matrix
                .into_iter()
                .map(|[price, quantity]| {
                    let price = price.parse().unwrap();
                    let quantity = quantity.parse().unwrap();
                    Order { price, quantity }
                })
                .collect::<Vec<Order>>()
        };

        let bids = [
            ["1336.28000000", "0.40950000"],
            ["1336.27000000", "0.35500000"],
            ["1336.22000000", "0.02150000"],
            ["1336.20000000", "0.35500000"],
            ["1336.12000000", "0.35500000"],
            ["1336.06000000", "0.35500000"],
            ["1335.77000000", "0.35500000"],
            ["1335.63000000", "0.49390000"],
            ["1335.62000000", "0.75330000"],
            ["1335.59000000", "0.62450000"],
        ];
        let asks = [
            ["1336.39000000", "0.35500000"],
            ["1336.41000000", "0.02150000"],
            ["1336.42000000", "0.38900000"],
            ["1336.44000000", "0.35500000"],
            ["1336.46000000", "0.35500000"],
            ["1336.60000000", "0.35500000"],
            ["1336.74000000", "0.15180000"],
            ["1336.75000000", "0.37730000"],
            ["1336.83000000", "1.00000000"],
            ["1336.92000000", "0.35500000"],
        ];

        let bids = convert_matrix_to_order_list(&bids);
        let asks = convert_matrix_to_order_list(&asks);

        let expected = OrderBook::new(bids, asks, "Binance");

        let result = BinanceStream::try_message_to_order_book(raw_json.into()).unwrap();

        assert_eq!(result, expected);
    }

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
