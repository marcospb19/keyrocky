use std::pin::Pin;

use futures::Stream;
use tokio::sync::broadcast::Sender;
use tokio_stream::wrappers::BroadcastStream;
use tonic::{transport::Server, Request, Response, Status};

use crate::{
    order_book::{Empty, OrderbookAggregator, OrderbookAggregatorService, Summary},
    Result,
};

type TonicResult<T> = Result<T, Status>;

pub async fn run_server(subscriber: Sender<Result<Summary, String>>, port: u16) -> Result<()> {
    let addr = format!("[::1]:{port}").parse().unwrap();

    let aggregator = OrderbookAggregatorChannel {
        channel_subscriber: subscriber,
    };

    Server::builder()
        .add_service(OrderbookAggregatorService::new(aggregator))
        .serve(addr)
        .await
        .unwrap();

    Ok(())
}

#[derive(Debug)]
pub struct OrderbookAggregatorChannel {
    channel_subscriber: Sender<Result<Summary, String>>,
}

#[tonic::async_trait]
impl OrderbookAggregator for OrderbookAggregatorChannel {
    type BookSummaryStream = Pin<Box<dyn Send + Stream<Item = TonicResult<Summary>>>>;

    async fn book_summary(
        &self,
        _request: Request<Empty>,
    ) -> TonicResult<Response<Self::BookSummaryStream>> {
        let receiver = self.channel_subscriber.subscribe();

        let stream = BroadcastStream::new(receiver);

        let stream = async_stream::stream! {
            for await message in stream {
                // Ignore lagged packets
                if let Ok(message) = message {
                    // Stream it
                    yield translate_channel_message(message);
                }
            }
        };

        Ok(Response::new(Box::pin(stream)))
    }
}

fn translate_channel_message(result: Result<Summary, String>) -> TonicResult<Summary> {
    result.map_err(|text| Status::internal(text))
}
