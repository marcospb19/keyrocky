/// Re-export items at the root crate for other modules.
pub use self::error::{Error, Result};

mod cli;
mod currencies;
mod error;
mod exchanges;
mod order_book;
mod server;
mod websocket;

use std::collections::HashMap;

use exchanges::{BinanceExchange, BitstampExchange};
use futures::{future, Stream, StreamExt};
use itertools::Itertools;
use merge_streams::MergeStreams;
use tokio::sync::broadcast;

use crate::{currencies::CurrencyPair, exchanges::ConnectToOrderBook, order_book::Summary};

const BROADCAST_QUEUE_CAPACITY: usize = 100;

#[tokio::main]
async fn main() {
    run().await.unwrap_or_else(|err| {
        eprintln!("{err}.");
        std::process::exit(1);
    });
}

async fn run() -> Result<()> {
    let (currency_pair, port) = cli::parse_arguments()?;

    let mut stream = build_aggregated_book_order(&currency_pair).await?;

    let (channel_subscriber, _) = broadcast::channel(BROADCAST_QUEUE_CAPACITY);
    let publisher = channel_subscriber.clone();

    // Consume the stream and transmit all summaries to the publisher
    tokio::spawn(async move {
        let stringify_error = |err| format!("{err}");

        while let Some(updated_summary) = stream.next().await {
            let summary = updated_summary.map_err(stringify_error);

            // Ignore send errors, nobody might be listening to this publisher now,
            // however, new listeners are spawned on-demand when requests are received.
            let _ = publisher.send(summary);
        }

        Ok(()) as Result<()>
    });

    server::run_server(channel_subscriber, port)
        .await
        .expect("cannot run server");

    Ok(())
}

/// Connects to exchanges and returns the aggregated book order stream.
async fn build_aggregated_book_order(
    currency_pair: &CurrencyPair,
) -> Result<impl Stream<Item = Result<Summary>>> {
    // Connect to exchange websockets, answer pings and parse summaries.
    let binance = BinanceExchange::connect_to_order_book(currency_pair).await?;
    let binance = websocket::answer_websocket_pings_adapter(binance);
    let binance = binance.map(|message| BinanceExchange::try_parse_summary(message?));

    let bitstamp = BitstampExchange::connect_to_order_book(currency_pair).await?;
    let bitstamp = websocket::answer_websocket_pings_adapter(bitstamp);
    let bitstamp = bitstamp.map(|message| BitstampExchange::try_parse_summary(message?));

    Ok(combine_streams(binance, bitstamp))
}

// Combine streams into a new stream, summaries are cached by
// the (hopefully) unique exchange names, and overwritten every
// time the same exchange updates it's latest summary.
fn combine_streams(
    left_stream: impl Stream<Item = Result<Summary>>,
    right_stream: impl Stream<Item = Result<Summary>>,
) -> impl Stream<Item = Result<Summary>> {
    let stream = (left_stream, right_stream).merge();
    stream.scan(
        HashMap::<String, Summary>::new(),
        |cached_summaries, next_summary| {
            let next_summary = match next_summary {
                Ok(next_summary) => next_summary,
                Err(err) => return future::ready(Some(Err(err))),
            };

            let cache_key = next_summary.asks[0].exchange.clone();
            cached_summaries.insert(cache_key, next_summary);

            let ordered_bids = cached_summaries
                .values()
                .flat_map(|summary| summary.bids.iter())
                .sorted_by(|left, right| left.price.partial_cmp(&right.price).unwrap().reverse());

            let ordered_asks = cached_summaries
                .values()
                .flat_map(|summary| summary.asks.iter())
                .sorted_by(|left, right| left.price.partial_cmp(&right.price).unwrap());

            let combined_order_book = Summary::new(
                ordered_bids.take(10).cloned().collect(),
                ordered_asks.take(10).cloned().collect(),
            );

            future::ready(Some(Ok(combined_order_book)))
        },
    )
}
