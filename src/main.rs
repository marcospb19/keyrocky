/// Re-export items at the root crate for other modules.
pub use self::error::{Error, Result};

mod cli;
mod currencies;
mod error;
mod exchanges;

use std::collections::HashMap;

use exchanges::{BinanceStream, BitstampStream};
use futures::{future, stream_select, Stream, StreamExt};
use itertools::Itertools;

use crate::exchanges::OrderBook;

#[tokio::main]
async fn main() {
    run().await.unwrap_or_else(|err| {
        eprintln!("{err}.");
        std::process::exit(1);
    });
}

async fn run() -> Result<()> {
    let (currency_pair, _port) = cli::parse_arguments()?;

    let binance_stream = BinanceStream::connect_and_subscribe(&currency_pair).await?;
    let bitstamp_stream = BitstampStream::connect_and_subscribe(&currency_pair).await?;

    let stream = combine_streams(binance_stream, bitstamp_stream).await?;

    debug_stream(stream).await
}

async fn combine_streams(
    binance_stream: BinanceStream,
    bitstamp_stream: BitstampStream,
) -> Result<impl Stream<Item = Result<OrderBook>>> {
    // This macro combines N streams into one that polls from all of them
    let stream = stream_select!(binance_stream, bitstamp_stream);
    let stream = stream.scan(
        HashMap::<String, OrderBook>::new(),
        |cached_data, order_book| {
            let order_book = match order_book {
                Ok(order_book) => order_book,
                Err(err) => return future::ready(Some(Err(err))),
            };

            let cache_key = order_book.asks[0].exchange.into();
            cached_data.insert(cache_key, order_book);

            let ordered_asks = cached_data
                .values()
                .flat_map(|order_book| order_book.asks.iter())
                .sorted_by(|left, right| left.price.cmp(&right.price))
                .take(10);

            let ordered_bids = cached_data
                .values()
                .flat_map(|order_book| order_book.bids.iter())
                .sorted_by(|left, right| left.price.cmp(&right.price).reverse())
                .take(10);

            let combined_order_book = OrderBook {
                asks: ordered_asks.cloned().collect(),
                bids: ordered_bids.cloned().collect(),
            };

            future::ready(Some(Ok(combined_order_book)))
        },
    );

    Ok(stream)
}

async fn debug_stream(stream: impl Stream<Item = Result<OrderBook>>) -> Result<()> {
    let mut stream = Box::pin(stream);

    while let Some(order_book) = stream.next().await {
        let order_book = order_book?;

        let binance = order_book
            .asks
            .iter()
            .chain(&order_book.bids)
            .filter(|order| order.exchange == "Binance")
            .count();
        let bitstamp = order_book
            .asks
            .iter()
            .chain(&order_book.bids)
            .filter(|order| order.exchange == "Bitstamp")
            .count();

        println!("- Binance: {binance:2} - Bitstamp: {bitstamp:2}");
    }

    Ok(())
}
