// Re-export proto definitions
pub use orderbook::{
    orderbook_aggregator_server::{
        OrderbookAggregator, OrderbookAggregatorServer as OrderbookAggregatorService,
    },
    Empty, Level, Summary,
};

mod orderbook {
    tonic::include_proto!("orderbook");
}

impl Summary {
    pub fn new(bids: Vec<Level>, asks: Vec<Level>) -> Self {
        assert!(bids.len() == 10);
        assert!(asks.len() == 10);
        let spread = asks[0].price - bids[0].price;
        Self { bids, asks, spread }
    }
}
