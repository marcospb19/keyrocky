use clap::Parser;

use crate::{
    currencies::{CurrencyPair, SUPPORTED_CURRENCY_PAIRS},
    Result,
};

pub fn parse_arguments() -> Result<(CurrencyPair, u16)> {
    let CliArgs {
        currency_pair,
        port,
    } = CliArgs::parse();

    let currency_pair = currency_pair.parse()?;

    Ok((currency_pair, port))
}

/// gRPC server that streams an order book for a currency pair.
#[derive(Parser, Debug)]
struct CliArgs {
    /// Currency pair for the order book.
    #[clap(
        default_value = "ETHBTC",
        possible_values = SUPPORTED_CURRENCY_PAIRS
    )]
    pub currency_pair: String,

    /// Port where the server will be served.
    #[clap(default_value = "50051")]
    pub port: u16,
}
