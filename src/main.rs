mod cli;
mod currencies;
mod error;
mod exchanges;

pub use error::{Error, Result};
use exchanges::bitstamp;
use futures::stream::StreamExt;
use tokio::io::{self, AsyncWriteExt};

#[tokio::main]
async fn main() {
    run().await.unwrap_or_else(|err| {
        eprintln!("{err}.");
        std::process::exit(1);
    });
}

async fn run() -> Result<()> {
    let (currency_pair, _port) = cli::parse_arguments()?;

    let mut bitstamp_stream = bitstamp::connect_and_subscribe(&currency_pair)
        .await
        .unwrap();

    while let Some(message) = bitstamp_stream.next().await {
        let data = message.unwrap().into_data();
        io::stdout().write_all(&data).await.unwrap();
        io::stdout().flush().await.unwrap();
    }

    Ok(())
}
