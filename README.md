# Keyrocky

This is a CLI tool that, given a pair of currencies (see
[list of supported currency pairs](#supported-currency-pairs))
reads book orders from `Binance` and `Bitstamp`, merges it in
a single stream, and serves it with a `gRPC` server stream.

# Installation

If you have `rustup` installed, run the following commands:

```sh
git clone https://github.com/marcospb19/keyrocky
cargo install --path keyrocky
```

# Usage

`keyrocky <CURRENCY_PAIR> <SERVER_PORT>`

# Supported Currency Pairs

|            |           |            |           |           |
|------------|-----------|------------|-----------|-----------|
| `AAVEBTC`  | `ADABTC`  | `ADAEUR`   | `ALGOBTC` | `APEEUR`  |
| `AUDIOBTC` | `AVAXEUR` | `BCHBTC`   | `BCHEUR`  | `BTCEUR`  |
| `BTCGBP`   | `BTCPAX`  | `BTCUSDC`  | `BTCUSDT` | `CHZEUR`  |
| `DOTEUR`   | `ENJEUR`  | `ETHBTC`   | `ETHEUR`  | `ETHGBP`  |
| `ETHPAX`   | `ETHUSDC` | `ETHUSDT`  | `FTMEUR`  | `GALAEUR` |
| `GRTEUR`   | `LINKBTC` | `LINKEUR`  | `LINKGBP` | `LTCBTC`  |
| `LTCEUR`   | `LTCGBP`  | `MATICEUR` | `NEAREUR` | `OMGBTC`  |
| `SHIBEUR`  | `SOLEUR`  | `SXPEUR`   | `UNIBTC`  | `UNIEUR`  |
| `USDCUSDT` | `WBTCBTC` | `XLMBTC`   | `XLMEUR`  | `XRPBTC`  |
| `XRPEUR`   | `XRPGBP`  | `XRPUSDT`  | `YFIEUR`  | _ |
