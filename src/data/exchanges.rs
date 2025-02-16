use crate::data::{
    liquidation::LiquidationData,
    market::MarketData,
    markprice::MarkPriceData,
    orderbook::book::{OrderbookData, OrderbookUpdateStream},
};
use tokio::sync::mpsc;

pub struct FutureChannels {
    pub ob_raw_in: mpsc::Receiver<OrderbookUpdateStream>,
    pub ob_raw_out: mpsc::Sender<OrderbookUpdateStream>,
    pub ob_mng_out: mpsc::Sender<OrderbookData>,
    pub agg_out: mpsc::Sender<MarketData>,
    pub liq_out: mpsc::Sender<LiquidationData>,
    pub mark_out: mpsc::Sender<MarkPriceData>,
}

pub struct SpotChannels {
    pub ob_raw_in: mpsc::Receiver<OrderbookUpdateStream>,
    pub ob_raw_out: mpsc::Sender<OrderbookUpdateStream>,
    pub ob_mng_out: mpsc::Sender<OrderbookData>,
    pub agg_out: mpsc::Sender<MarketData>,
}
