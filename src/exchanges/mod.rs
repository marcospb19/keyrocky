pub use binance::BinanceStream;
pub use bitstamp::BitstampStream;

pub mod binance;
pub mod bitstamp;

use bigdecimal::BigDecimal;
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

type WebSocket = WebSocketStream<MaybeTlsStream<TcpStream>>;

#[derive(Debug, PartialEq, Eq)]
pub struct Order {
    price: BigDecimal,
    quantity: BigDecimal,
}

#[derive(Debug, PartialEq, Eq)]
pub struct OrderBook {
    /// The best ten bids.
    best_bids: Vec<Order>,
    /// The best ten asks.
    best_asks: Vec<Order>,
    /// Name of the exchange which this book originated from.
    origin: &'static str,
}

impl OrderBook {
    pub fn new(best_bids: Vec<Order>, best_asks: Vec<Order>, origin: &'static str) -> Self {
        Self {
            best_bids,
            best_asks,
            origin,
        }
    }
}
