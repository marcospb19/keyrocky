/// Re-export items at the root crate for other modules.
pub use self::error::{Error, Result};

mod cli;
mod currencies;
mod error;
mod exchanges;

use std::collections::HashMap;

use async_stream::*;
use exchanges::{binance, bitstamp};
use futures::{future, Sink, SinkExt, Stream, StreamExt};
use itertools::Itertools;
use merge_streams::MergeStreams;
use tungstenite::Message;

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

    let binance_stream = binance::connect_and_subscribe(&currency_pair)
        .await
        .map(answer_websocket_pings)?
        .map(|message| binance::try_message_to_order_book(message?));

    let bitstamp_stream = bitstamp::connect_and_subscribe(&currency_pair)
        .await
        .map(answer_websocket_pings)?
        .map(|message| bitstamp::try_message_to_order_book(message?));

    let stream = combine_streams(binance_stream, bitstamp_stream);
    debug_stream(stream).await?;

    Ok(())
}

fn combine_streams(
    left_stream: impl Stream<Item = Result<OrderBook>>,
    right_stream: impl Stream<Item = Result<OrderBook>>,
) -> impl Stream<Item = Result<OrderBook>> {
    let stream = (left_stream, right_stream).merge();
    stream.scan(
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
    )
}

fn answer_websocket_pings<W>(websocket: W) -> impl Stream<Item = Result<String>>
where
    W: Sink<Message> + Stream<Item = tungstenite::Result<Message>> + Unpin,
    Error: From<W::Error>,
{
    let (mut sink, stream) = websocket.split();

    try_stream! {
        for await message in stream {
            let message = message?;

            match message {
                Message::Text(text) => yield text,
                Message::Ping(data) => sink.send(Message::Pong(data)).await?,
                _ => {},
            }
        }
    }
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
