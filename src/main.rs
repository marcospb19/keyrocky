mod cli;
mod currencies;
mod error;
mod exchanges;

pub use error::{Error, Result};
use exchanges::{binance, bitstamp};
use futures::stream::StreamExt;
use tungstenite::Message;

#[tokio::main]
async fn main() {
    run().await.unwrap_or_else(|err| {
        eprintln!("{err}.");
        std::process::exit(1);
    });
}

async fn run() -> Result<()> {
    let (currency_pair, _port) = cli::parse_arguments()?;

    let mut bitstamp_stream = bitstamp::connect_and_subscribe(&currency_pair).await?;
    let mut binance_stream = binance::connect_and_subscribe(&currency_pair).await?;

    let bitstamp_log_stream = async {
        while let Some(message) = bitstamp_stream.next().await {
            debug_message("bitstamp", message.unwrap());
        }
    };
    let binance_log_stream = async {
        while let Some(message) = binance_stream.next().await {
            debug_message("binance", message.unwrap());
        }
    };

    futures::join!(bitstamp_log_stream, binance_log_stream);

    Ok(())
}

fn debug_message(from: &str, message: Message) {
    let json_raw = message.into_text().unwrap();
    let json_value: serde_json::Value = serde_json::from_str(&json_raw).unwrap();
    let json_formatted = serde_json::to_string_pretty(&json_value).unwrap();

    println!("{from}: {json_formatted}");
}
