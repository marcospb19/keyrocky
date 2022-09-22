use bigdecimal::ParseBigDecimalError;

pub type Result<T, E = self::Error> = std::result::Result<T, E>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Currency error: currency pair '{0}' is invalid")]
    CurrencyPairBadFormat(String),
    #[error("{0} stream Error: stream was expected to send at least 10 {1}")]
    NotEnoughOrders(String, String),
    #[error("WebSocket error: {0}")]
    TungsteniteError(#[from] tungstenite::error::Error),
    #[error("Failed to parse integer: {0}")]
    BigdecimalError(#[from] ParseBigDecimalError),
}
