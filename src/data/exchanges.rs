use crate::data::{
    depth::OrderbookUpdateStream, liquidation::LiquidationData, market::MarketData,
    markprice::MarkPriceData,
};
use tokio::sync::mpsc;

pub struct FutureDataChannels {
    pub ob_out: mpsc::Sender<OrderbookUpdateStream>,
    pub agg_out: mpsc::Sender<MarketData>,
    pub liq_out: mpsc::Sender<LiquidationData>,
    pub mark_out: mpsc::Sender<MarkPriceData>,
}

pub struct SpotDataChannels {
    pub ob_out: mpsc::Sender<OrderbookUpdateStream>,
    pub agg_out: mpsc::Sender<MarketData>,
}
