pub mod future;
pub mod market_state;
pub mod spot;

use crate::data::{depth, market};
use crate::prism::orderbook::Orderbook;
use log::debug;
use market_state::MarketState;
use tokio::sync::mpsc;

pub struct Core<Rx> {
    // Data Channel
    ob: mpsc::Receiver<depth::OrderbookUpdateStream>,
    agg: mpsc::Receiver<market::MarketData>,

    additional: Rx,

    // Market State
    pub market_state: MarketState,
    // Orderbook
    pub total_orderbook: Orderbook,
    // pub filtered_orderbook: Orderbook,
    // Bars
}

impl<Rx> Core<Rx> {
    pub fn debug(&self) {
        debug!("Market State: {:?}", self.market_state);
        debug!(
            "Total Orderbook Best Ask: {:?}",
            self.total_orderbook.best_ask()
        );
        debug!(
            "Total Orderbook Best Bid: {:?}",
            self.total_orderbook.best_bid()
        );
    }
}
