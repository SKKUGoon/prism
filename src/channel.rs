use crate::data::{
    depth::OrderbookUpdateStream, liquidation::LiquidationData, market::MarketData,
    markprice::MarkPriceData,
};
use tokio::sync::mpsc;

pub struct DataChannelPairs<Channel> {
    // Data Channel: Orderbook -> Engine
    pub ob: (
        mpsc::Sender<OrderbookUpdateStream>,
        mpsc::Receiver<OrderbookUpdateStream>,
    ),
    // Data Channel: Aggtrade -> Engine
    pub agg: (mpsc::Sender<MarketData>, mpsc::Receiver<MarketData>),
    // Data Channel for futures(etc)
    // Data Channel: Mark Price -> Engine,
    // Data Channel: Liquidation -> Engine
    pub additional: Channel,
}

pub struct Spot;
pub struct Future {
    // Mark Price -> Engine
    pub mark: (mpsc::Sender<MarkPriceData>, mpsc::Receiver<MarkPriceData>),
    // Liquidation -> Engine
    pub liq: (
        mpsc::Sender<LiquidationData>,
        mpsc::Receiver<LiquidationData>,
    ),
}

pub type SpotChannel = DataChannelPairs<Spot>;
pub type FutureChannel = DataChannelPairs<Future>;

impl SpotChannel {
    pub fn new(max_capacity: usize) -> Self {
        let (tx_ob_raw, rx_ob_raw) = mpsc::channel(max_capacity);
        let (tx_agg, rx_agg) = mpsc::channel(max_capacity);

        Self {
            ob: (tx_ob_raw, rx_ob_raw),
            agg: (tx_agg, rx_agg),
            additional: Spot,
        }
    }
}

impl FutureChannel {
    pub fn new(max_capacity: usize) -> Self {
        let (tx_ob_raw, rx_ob_raw) = mpsc::channel(max_capacity);
        let (tx_agg, rx_agg) = mpsc::channel(max_capacity);
        let (tx_mark, rx_mark) = mpsc::channel(max_capacity);
        let (tx_liq, rx_liq) = mpsc::channel(max_capacity);

        Self {
            ob: (tx_ob_raw, rx_ob_raw),
            agg: (tx_agg, rx_agg),
            additional: Future {
                mark: (tx_mark, rx_mark),
                liq: (tx_liq, rx_liq),
            },
        }
    }
}
