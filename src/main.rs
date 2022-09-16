use futures::stream::StreamExt;
use tokio::io::{self, AsyncWriteExt};
use tokio_tungstenite as websocket;

const BITSTAMP_WEBSOCKET_URL: &str = "wss://ws.bitstamp.net";

#[tokio::main]
async fn main() {
    let (mut read_stream, _response) = websocket::connect_async(BITSTAMP_WEBSOCKET_URL)
        .await
        .unwrap();

    println!("BitStamp handshake success!!!");

    while let Some(message) = read_stream.next().await {
        let data = message.unwrap().into_data();
        io::stdout().write_all(&data).await.unwrap();
    }
}
