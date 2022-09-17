pub mod bitstamp;

use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

pub type ExchangeStream = WebSocketStream<MaybeTlsStream<TcpStream>>;
