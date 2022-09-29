use std::str::FromStr;

use crate::Error;

/// A curency pair like "ETHBTC".
pub struct CurrencyPair(String);

impl CurrencyPair {
    /// Extracts the inner string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for CurrencyPair {
    type Err = Error;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let is_ascii_string = text.is_ascii();
        let has_expected_size = text.len() == 6;
        let is_alphabetic_text = text.chars().all(char::is_alphabetic);

        let is_valid = is_ascii_string && has_expected_size && is_alphabetic_text;

        if !is_valid {
            return Err(Error::CurrencyPairBadFormat(text.to_owned()));
        }

        Ok(CurrencyPair(text.to_owned()))
    }
}

/// All currency pairs supported by Binance and Bitstamp.
pub const SUPPORTED_CURRENCY_PAIRS: [&str; 49] = [
    "AAVEBTC", "ADABTC", "ADAEUR", "ALGOBTC", "APEEUR", "AUDIOBTC", "AVAXEUR", "BCHBTC", "BCHEUR",
    "BTCEUR", "BTCGBP", "BTCPAX", "BTCUSDC", "BTCUSDT", "CHZEUR", "DOTEUR", "ENJEUR", "ETHBTC",
    "ETHEUR", "ETHGBP", "ETHPAX", "ETHUSDC", "ETHUSDT", "FTMEUR", "GALAEUR", "GRTEUR", "LINKBTC",
    "LINKEUR", "LINKGBP", "LTCBTC", "LTCEUR", "LTCGBP", "MATICEUR", "NEAREUR", "OMGBTC", "SHIBEUR",
    "SOLEUR", "SXPEUR", "UNIBTC", "UNIEUR", "USDCUSDT", "WBTCBTC", "XLMBTC", "XLMEUR", "XRPBTC",
    "XRPEUR", "XRPGBP", "XRPUSDT", "YFIEUR",
];
