use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::{
    currencies::CurrencyPair,
    exchanges::ConnectToOrderBook,
    order_book::{Level, Summary},
    Error, Result,
};

const BINANCE_WEBSOCKET_BASE_URL: &str = "wss://stream.binance.com:9443/ws";
const EXCHANGE_NAME: &str = "Binance";

pub struct BinanceExchange;

impl ConnectToOrderBook for BinanceExchange {
    type SubscribeMessage = BinanceSubscribeMessage;

    fn connect_url(currency_pair: &CurrencyPair) -> String {
        let suffix = currency_pair.as_str();
        format!("{BINANCE_WEBSOCKET_BASE_URL}/{suffix}")
    }

    fn subscribe_message(currency_pair: &CurrencyPair) -> Self::SubscribeMessage {
        BinanceSubscribeMessage::new(currency_pair)
    }
}

impl BinanceExchange {
    pub fn try_parse_summary(message: String) -> Result<Summary> {
        let BinanceRawLevelBook { mut bids, mut asks } = serde_json::from_str(&message).unwrap();

        if asks.len() < 10 {
            return Err(Error::NotEnoughOrders(EXCHANGE_NAME.into(), "bids".into()));
        }

        if asks.len() < 10 {
            return Err(Error::NotEnoughOrders(EXCHANGE_NAME.into(), "asks".into()));
        }

        asks.resize_with(10, || unreachable!());
        bids.resize_with(10, || unreachable!());

        let array_into_level = |array: RawLevel| -> Result<Level, <f64 as FromStr>::Err> {
            let [price, amount] = array;

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

type RawLevel = [String; 2];

#[derive(Deserialize)]
struct BinanceRawLevelBook {
    bids: Vec<RawLevel>,
    asks: Vec<RawLevel>,
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
                .map(|[price, amount]| {
                    let price = price.parse().unwrap();
                    let amount = amount.parse().unwrap();
                    Level {
                        price,
                        amount,
                        exchange: "Binance".to_string(),
                    }
                })
                .collect::<Vec<Level>>()
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

        let expected = Summary::new(bids, asks);

        let result = BinanceExchange::try_parse_summary(raw_json.into()).unwrap();

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
