//! Websocket adapters and definitions.

use async_stream::try_stream;
use futures::{Sink, SinkExt, Stream, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tungstenite::Message;

use crate::{Error, Result};

pub type WebSocket = WebSocketStream<MaybeTlsStream<TcpStream>>;

pub async fn websocket_connect(url: impl AsRef<str>) -> Result<WebSocket> {
    let (websocket, _) = connect_async(url.as_ref()).await?;
    Ok(websocket)
}

/// Wraps a websocket in a stream that answers for pings.
pub fn answer_websocket_pings_adapter<W>(websocket: W) -> impl Stream<Item = Result<String>>
where
    W: Sink<Message> + Stream<Item = tungstenite::Result<Message>>,
    Error: From<W::Error>,
{
    let (mut sink, stream) = websocket.split();

    try_stream! {
        for await message in stream {
            let message = message?;

            match message {
                Message::Text(text) => yield text,
                Message::Ping(data) => sink.send(Message::Pong(data)).await?,
                _ => {},
            }
        }
    }
}
