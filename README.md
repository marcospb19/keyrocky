# Keyrocky

This is a CLI tool that, given a pair of currencies (see
[list of supported currency pairs](#supported-currency-pairs))
reads book orders from `Binance` and `Bitstamp`, merges it in
a single stream, and serves it with a `gRPC` server stream.

## Installation

If you have `rustup` installed, run the following commands:

```sh
git clone https://github.com/marcospb19/keyrocky
cargo install --path keyrocky
```

## Usage

`keyrocky <CURRENCY_PAIR> <SERVER_PORT>`

## Help message

![image](https://user-images.githubusercontent.com/38900226/192727476-4dc4f40d-73d8-46d3-9817-569e46a4e9f1.png)

### Supported Currency Pairs

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


## Missing features

- Retry to reconnect to websockets that have failed.
- Ignore websockets message errors instead of aborting.
- Logs.
- Better error treatment.
