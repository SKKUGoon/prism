use crate::data::{
    liquidation::LiquidationData,
    market::MarketData,
    markprice::MarkPriceData,
    orderbook::{OrderbookData, OrderbookUpdateStream},
};
use tokio::sync::mpsc;

pub struct FutureDataChannels {
    pub ob_raw_in: mpsc::Receiver<OrderbookUpdateStream>,
    pub ob_raw_out: mpsc::Sender<OrderbookUpdateStream>,
    pub ob_mng_out: mpsc::Sender<OrderbookData>,
    pub agg_out: mpsc::Sender<MarketData>,
    pub liq_out: mpsc::Sender<LiquidationData>,
    pub mark_out: mpsc::Sender<MarkPriceData>,
}

pub struct SpotDataChannels {
    pub ob_raw_in: mpsc::Receiver<OrderbookUpdateStream>,
    pub ob_raw_out: mpsc::Sender<OrderbookUpdateStream>,
    pub ob_mng_out: mpsc::Sender<OrderbookData>,
    pub agg_out: mpsc::Sender<MarketData>,
}
