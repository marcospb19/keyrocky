use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::{
    currencies::CurrencyPair,
    exchanges::ConnectToOrderBook,
    order_book::{Level, Summary},
    Error, Result,
};

const BITSTAMP_WEBSOCKET_URL: &str = "wss://ws.bitstamp.net";
const EXCHANGE_NAME: &str = "Bitstamp";

pub struct BitstampExchange;

impl ConnectToOrderBook for BitstampExchange {
    type SubscribeMessage = BitstampSubscribeMessage;

    fn connect_url(_currency_pair: &CurrencyPair) -> String {
        BITSTAMP_WEBSOCKET_URL.into()
    }

    fn subscribe_message(currency_pair: &CurrencyPair) -> Self::SubscribeMessage {
        BitstampSubscribeMessage::new(currency_pair)
    }
}

impl BitstampExchange {
    pub fn try_parse_summary(message: String) -> Result<Summary> {
        let BitstampRawSummary {
            data: BitstampSummaryData { mut bids, mut asks },
        } = serde_json::from_str(&message).unwrap();

        if asks.len() < 10 {
            return Err(Error::NotEnoughOrders(EXCHANGE_NAME.into(), "bids".into()));
        }

        if asks.len() < 10 {
            return Err(Error::NotEnoughOrders(EXCHANGE_NAME.into(), "asks".into()));
        }

        asks.resize_with(10, || unreachable!());
        bids.resize_with(10, || unreachable!());

        let array_into_level = |array: RawLevel| -> Result<Level, <f64 as FromStr>::Err> {
            let [price, amount, _identifier] = array;

            Ok(Level {
                price: price.parse()?,
                amount: amount.parse()?,
                exchange: EXCHANGE_NAME.to_string(),
            })
        };

        let bids = bids
            .into_iter()
            .map(array_into_level)
            .collect::<Result<_, _>>()?;

        let asks = asks
            .into_iter()
            .map(array_into_level)
            .collect::<Result<_, _>>()?;

        Ok(Summary::new(bids, asks))
    }
}

type RawLevel = [String; 3];

#[derive(Deserialize)]
struct BitstampRawSummary {
    data: BitstampSummaryData,
}

#[derive(Deserialize)]
struct BitstampSummaryData {
    bids: Vec<RawLevel>,
    asks: Vec<RawLevel>,
}

#[derive(Serialize)]
pub struct BitstampSubscribeMessage {
    event: String,
    data: BitstampChannelInformation,
}

impl BitstampSubscribeMessage {
    pub fn new(currency_pair: &CurrencyPair) -> Self {
        Self {
            event: "bts:subscribe".into(),
            data: BitstampChannelInformation::new(currency_pair),
        }
    }
}

#[derive(Serialize)]
pub struct BitstampChannelInformation {
    channel: String,
}

impl BitstampChannelInformation {
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

        let result = BitstampExchange::try_parse_summary(raw_json.into()).unwrap();

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
