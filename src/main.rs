/// Re-export items at the root crate for other modules.
pub use self::error::{Error, Result};

mod cli;
mod currencies;
mod error;
mod exchanges;

use exchanges::{BinanceStream, BitstampStream};
use futures::{Stream, StreamExt};

use crate::exchanges::OrderBook;

#[tokio::main]
async fn main() {
    run().await.unwrap_or_else(|err| {
        eprintln!("{err}.");
        std::process::exit(1);
    });
}

#[allow(unused)]
async fn run() -> Result<()> {
    let (currency_pair, _port) = cli::parse_arguments()?;

    let mut bitstamp_stream = BitstampStream::connect_and_subscribe(&currency_pair).await?;
    let mut binance_stream = BinanceStream::connect_and_subscribe(&currency_pair).await?;

    let bitstamp_order_books = log_order_books(bitstamp_stream);
    let binance_order_books = log_order_books(binance_stream);

    futures::try_join!(bitstamp_order_books, binance_order_books);

    Ok(())
}

async fn log_order_books<S>(mut stream: S) -> Result<()>
where
    S: Stream<Item = Result<OrderBook>> + Unpin,
{
    while let Some(order_book) = stream.next().await {
        let order_book = order_book?;
        dbg!(order_book);
    }

    Ok(())
}
