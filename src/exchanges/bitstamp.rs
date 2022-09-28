use std::str::FromStr;

use bigdecimal::BigDecimal;
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio_tungstenite::connect_async;
use tungstenite::Message;

use crate::{
    currencies::CurrencyPair,
    exchanges::WebSocket,
    order_book::{Level, Summary},
    Error, Result,
};

const BITSTAMP_WEBSOCKET_URL: &str = "wss://ws.bitstamp.net";
const EXCHANGE_NAME: &str = "Bitstamp";

pub async fn connect_and_subscribe(currency_pair: &CurrencyPair) -> Result<WebSocket> {
    let (mut websocket, _) = connect_async(BITSTAMP_WEBSOCKET_URL).await?;

    let message = BitstampSubscribeMessage::new(currency_pair);
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

    Ok(websocket)
}

pub fn try_message_to_order_book(message: String) -> Result<Summary> {
    let BitstampRawOrderBook {
        data: BitstampOrderBookData { mut bids, mut asks },
    } = serde_json::from_str(&message).unwrap();

    if asks.len() < 10 {
        return Err(Error::NotEnoughOrders(EXCHANGE_NAME.into(), "bids".into()));
    }

    if asks.len() < 10 {
        return Err(Error::NotEnoughOrders(EXCHANGE_NAME.into(), "asks".into()));
    }

    asks.resize_with(10, || unreachable!());
    bids.resize_with(10, || unreachable!());

    let array_into_order = |array: RawOrder| -> Result<Level, <BigDecimal as FromStr>::Err> {
        let [price, amount, _identifier] = array;

        let price = price.parse()?;
        let amount = amount.parse()?;

        Ok(Level {
            price,
            amount,
            exchange: EXCHANGE_NAME.to_string(),
        })
    };

    let bids = bids
        .into_iter()
        .map(array_into_order)
        .collect::<Result<_, _>>()?;

    let asks = asks
        .into_iter()
        .map(array_into_order)
        .collect::<Result<_, _>>()?;

    Ok(Summary::new(bids, asks))
}

type RawOrder = [String; 3];

#[derive(Deserialize)]
struct BitstampRawOrderBook {
    data: BitstampOrderBookData,
}

#[derive(Deserialize)]
struct BitstampOrderBookData {
    bids: Vec<RawOrder>,
    asks: Vec<RawOrder>,
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
    fn test_bitstamp_deserializing_order_book() {
        let raw_json = include_str!("../../test_data/bitstamp_order_book_update_message.json");

        let convert_matrix_to_order_list = |matrix: &[[&str; 2]]| {
            matrix
                .into_iter()
                .map(|[price, amount]| {
                    let price = price.parse().unwrap();
                    let amount = amount.parse().unwrap();
                    Level {
                        price,
                        amount,
                        exchange: "Bitstamp".to_string(),
                    }
                })
                .collect::<Vec<Level>>()
        };

        let asks = [
            ["1377.8", "3.98824761"],
            ["1377.8", "5.43831838"],
            ["1377.8", "3.98821949"],
            ["1377.9", "3.88262088"],
            ["1378.0", "1.70000000"],
            ["1378.0", "1.98341909"],
            ["1378.0", "1.70000000"],
            ["1378.1", "14.49953937"],
            ["1378.2", "1.70000000"],
            ["1378.4", "1.70000000"],
        ];

        let bids = [
            ["1377.2", "3.98969157"],
            ["1377.2", "5.44057836"],
            ["1377.2", "1.70000000"],
            ["1377.1", "5.44073049"],
            ["1377.0", "1.70000000"],
            ["1376.9", "2.12552611"],
            ["1376.8", "3.93983631"],
            ["1376.8", "1.70000000"],
            ["1376.7", "21.76903022"],
            ["1376.4", "5.23000000"],
        ];

        let bids = convert_matrix_to_order_list(&bids);
        let asks = convert_matrix_to_order_list(&asks);

        let expected = Summary::new(bids, asks);

        let result = try_message_to_order_book(raw_json.into()).unwrap();

        assert_eq!(result, expected);
    }

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
