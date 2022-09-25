pub use binance::BinanceStream;
pub use bitstamp::BitstampStream;

pub mod binance;
pub mod bitstamp;

use bigdecimal::BigDecimal;
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

type WebSocket = WebSocketStream<MaybeTlsStream<TcpStream>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Order {
    pub price: BigDecimal,
    pub quantity: BigDecimal,
    pub exchange: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderBook {
    /// The best ten bids.
    pub bids: Vec<Order>,
    /// The best ten asks.
    pub asks: Vec<Order>,
}

impl OrderBook {
    pub fn new(bids: Vec<Order>, asks: Vec<Order>) -> Self {
        Self { bids, asks }
    }
}
