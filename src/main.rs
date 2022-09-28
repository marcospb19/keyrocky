/// Re-export items at the root crate for other modules.
pub use self::error::{Error, Result};

mod cli;
mod currencies;
mod error;
mod exchanges;
mod order_book;
mod server;

use std::collections::HashMap;

use async_stream::*;
use exchanges::{binance, bitstamp};
use futures::{future, Sink, SinkExt, Stream, StreamExt};
use itertools::Itertools;
use merge_streams::MergeStreams;
use tokio::sync::broadcast;
use tungstenite::Message;

use crate::{currencies::CurrencyPair, order_book::Summary};

#[tokio::main]
async fn main() {
    run().await.unwrap_or_else(|err| {
        eprintln!("{err}.");
        std::process::exit(1);
    });
}

async fn run() -> Result<()> {
    let (currency_pair, port) = cli::parse_arguments()?;

    let mut stream = build_aggregated_stream(&currency_pair).await?;

    let (tx, _rx) = broadcast::channel::<Result<Summary, String>>(100);

    let orderbook_broadcast = tx.clone();
    tokio::spawn(async move {
        while let Some(updated_order_book) = stream.next().await {
            match updated_order_book {
                Ok(order_book) => {
                    orderbook_broadcast.send(Ok(order_book)).unwrap();
                }
                Err(error) => {
                    let error_message = format!("{error}");
                    orderbook_broadcast.send(Err(error_message)).unwrap();
                }
            }
        }
        Ok(()) as Result<()>
    });

    server::run_server(tx, port)
        .await
        .expect("cannot run server");

    Ok(())
}

pub async fn build_aggregated_stream(
    currency_pair: &CurrencyPair,
) -> Result<impl Stream<Item = Result<Summary>>> {
    let binance_stream = binance::connect_and_subscribe(currency_pair)
        .await
        .map(answer_websocket_pings_adapter)?
        .map(|message| binance::try_message_to_order_book(message?));

    let bitstamp_stream = bitstamp::connect_and_subscribe(currency_pair)
        .await
        .map(answer_websocket_pings_adapter)?
        .map(|message| bitstamp::try_message_to_order_book(message?));

    Ok(combine_streams(binance_stream, bitstamp_stream))
}

fn combine_streams(
    left_stream: impl Stream<Item = Result<Summary>>,
    right_stream: impl Stream<Item = Result<Summary>>,
) -> impl Stream<Item = Result<Summary>> {
    let stream = (left_stream, right_stream).merge();
    stream.scan(
        HashMap::<String, Summary>::new(),
        |cached_data, order_book| {
            let order_book = match order_book {
                Ok(order_book) => order_book,
                Err(err) => return future::ready(Some(Err(err))),
            };

            let cache_key = order_book.asks[0].exchange.clone();
            cached_data.insert(cache_key, order_book);

            let ordered_bids = cached_data
                .values()
                .flat_map(|order_book| order_book.bids.iter())
                .sorted_by(|left, right| left.price.partial_cmp(&right.price).unwrap().reverse())
                .take(10);

            let ordered_asks = cached_data
                .values()
                .flat_map(|order_book| order_book.asks.iter())
                .sorted_by(|left, right| left.price.partial_cmp(&right.price).unwrap())
                .take(10);

            let combined_order_book = Summary::new(
                ordered_bids.cloned().collect(),
                ordered_asks.cloned().collect(),
            );

            future::ready(Some(Ok(combined_order_book)))
        },
    )
}

fn answer_websocket_pings_adapter<W>(websocket: W) -> impl Stream<Item = Result<String>>
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
