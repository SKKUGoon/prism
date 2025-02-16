use crate::data::{
    liquidation::LiquidationData,
    market::MarketData,
    markprice::MarkPriceData,
    orderbook::book::{OrderbookData, OrderbookUpdateStream},
};
use crate::prism::stream::FeatureProcessed;
use tokio::sync::mpsc;

pub struct OrderbookChannel {
    // Orderbook -> Orderbook Processor
    pub raw: (
        mpsc::Sender<OrderbookUpdateStream>,
        mpsc::Receiver<OrderbookUpdateStream>,
    ),
    // Orderbook Processor -> Engine
    pub mng: (mpsc::Sender<OrderbookData>, mpsc::Receiver<OrderbookData>),
}

pub struct DataChannelPairs<Channel> {
    // Data Channel: Orderbook -> Engine
    pub ob: OrderbookChannel,
    // Data Channel: Aggtrade -> Engine
    pub agg: (mpsc::Sender<MarketData>, mpsc::Receiver<MarketData>),
    // Data Channel: Mark Price -> Engine, Liquidation -> Engine
    pub additional: Channel,
}

pub struct SystemChannelPairs {
    // Engine Channel: Engine -> Executor
    pub exec: (
        mpsc::Sender<FeatureProcessed>,
        mpsc::Receiver<FeatureProcessed>,
    ),
    pub db: (
        mpsc::Sender<FeatureProcessed>,
        mpsc::Receiver<FeatureProcessed>,
    ),
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
        let (tx_ob_mng, rx_ob_mng) = mpsc::channel(max_capacity);
        let (tx_agg, rx_agg) = mpsc::channel(max_capacity);

        Self {
            ob: OrderbookChannel {
                raw: (tx_ob_raw, rx_ob_raw),
                mng: (tx_ob_mng, rx_ob_mng),
            },
            agg: (tx_agg, rx_agg),
            additional: Spot,
        }
    }
}

impl FutureChannel {
    pub fn new(max_capacity: usize) -> Self {
        let (tx_ob_raw, rx_ob_raw) = mpsc::channel(max_capacity);
        let (tx_ob_mng, rx_ob_mng) = mpsc::channel(max_capacity);
        let (tx_agg, rx_agg) = mpsc::channel(max_capacity);
        let (tx_mark, rx_mark) = mpsc::channel(max_capacity);
        let (tx_liq, rx_liq) = mpsc::channel(max_capacity);

        Self {
            ob: OrderbookChannel {
                raw: (tx_ob_raw, rx_ob_raw),
                mng: (tx_ob_mng, rx_ob_mng),
            },
            agg: (tx_agg, rx_agg),
            additional: Future {
                mark: (tx_mark, rx_mark),
                liq: (tx_liq, rx_liq),
            },
        }
    }
}

impl SystemChannelPairs {
    pub fn new(max_capacity: usize) -> Self {
        let (tx_exec, rx_exec) = mpsc::channel(max_capacity);
        let (tx_db, rx_db) = mpsc::channel(max_capacity);

        Self {
            exec: (tx_exec, rx_exec),
            db: (tx_db, rx_db),
        }
    }
}
