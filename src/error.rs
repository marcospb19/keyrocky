pub type Result<T> = std::result::Result<T, self::Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Currency error: currency pair '{0}' is invalid")]
    CurrencyPairBadFormat(String),
    #[error("WebSocket error: {0}")]
    TungsteniteError(#[from] tungstenite::error::Error),
}
