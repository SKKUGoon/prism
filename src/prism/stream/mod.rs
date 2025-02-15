pub mod future;
pub mod spot;
use crate::data::{
    liquidation::LiquidationData, market::MarketData, markprice::MarkPriceData,
    orderbook::book::OrderbookData,
};
use crate::prism::bar::{DollarImbalanceBar, TickImbalanceBar, VolumeImbalanceBar};
use tokio::sync::mpsc::{Receiver, Sender};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FeatureProcessed {
    // What event is this?
    pub event_type: Option<String>,
    // When the websocket event is received
    pub event_time: u64,
    // When the feature is processed
    pub processed_time: u64,
    // When the trade is executed
    pub trade_time: u64,

    // Liquidation Dollar Volume - Only Future
    pub liquidation_side: String,
    pub liquidation_dvolume: f32,

    // Mark Price - Only Future
    pub mark_price: f32,
    pub funding_rate: f32,
    pub next_funding_time: u64,

    // f for Future, s for Spot
    pub source: String,

    pub price: f32,
    pub maker_quantity: f32,
    pub taker_quantity: f32,

    // Best ask - best bid
    pub ob_spread: f32,
    // Orderbook imbalance
    pub obi: f32,
    // Ranged Orderbook imbalance
    pub obi_range: (f32, f32),

    // Latest processed tick imbalance bar (Historical)
    pub tick_imbalance_bar: TickImbalanceBar,
    // Latest processed volume imbalance bar (Historical)
    pub volume_imbalance_bar: VolumeImbalanceBar,
    // Latest processed dollar imbalance bar (Historical)
    pub dollar_imbalance_bar: DollarImbalanceBar,

    // Real time tick imbalance information
    pub tick_imbalance: f32,
    // Real time tick imbalance threshold
    pub tick_imbalance_thres: f32,
    // Real time tick imbalance VWAP
    pub tick_imbalance_vwap: f32,

    // Real time volume imbalance information
    pub volume_imbalance: f32,
    // Real time volume imbalance threshold
    pub volume_imbalance_thres: f32,
    // Real time volume imbalance VWAP
    pub volume_imbalance_vwap: f32,
    // Real time volume imbalance CVD
    pub volume_imbalance_cvd: f32,

    // Real time dollar imbalance information
    pub dollar_imbalance: f32,
    // Real time dollar imbalance threshold
    pub dollar_imbalance_thres: f32,
    // Real time dollar imbalance VWAP
    pub dollar_imbalance_vwap: f32,
    // Real time dollar imbalance CVD
    pub dollar_imbalance_cvd: f32,

    // Whether the genesis tick imbalance bar has been created
    pub tick_imbalance_bar_init: bool,
    // Whether the genesis volume imbalance bar has been created
    pub volume_imbalance_bar_init: bool,
    // Whether the genesis dollar imbalance bar has been created
    pub dollar_imbalance_bar_init: bool,
}

impl FeatureProcessed {
    pub fn new() -> Self {
        Self {
            // Time
            event_type: None,
            event_time: 0,
            processed_time: 0,
            trade_time: 0,

            // Liquidation Dollar Volume
            liquidation_side: "".to_string(),
            liquidation_dvolume: 0.0,

            // Mark Price
            mark_price: 0.0,
            funding_rate: 0.0,
            next_funding_time: 0,

            source: "".to_string(),
            price: -f32::INFINITY,
            maker_quantity: 0.0,
            taker_quantity: 0.0,

            ob_spread: 0.0,
            obi: -f32::INFINITY, // Should be inside -1 < obi < 1
            obi_range: (0.0, 0.0),

            // Information bars
            tick_imbalance_bar: TickImbalanceBar::new(), // Initialize single tick imbalance bar
            tick_imbalance_bar_init: false,
            tick_imbalance: 0.0,
            tick_imbalance_vwap: 0.0,
            tick_imbalance_thres: 0.0,

            volume_imbalance_bar: VolumeImbalanceBar::new(),
            volume_imbalance_bar_init: false,
            volume_imbalance: 0.0,
            volume_imbalance_vwap: 0.0,
            volume_imbalance_thres: 0.0,
            volume_imbalance_cvd: 0.0,

            dollar_imbalance_bar: DollarImbalanceBar::new(),
            dollar_imbalance_bar_init: false,
            dollar_imbalance: 0.0,
            dollar_imbalance_vwap: 0.0,
            dollar_imbalance_thres: 0.0,
            dollar_imbalance_cvd: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FeatureInProgress {
    // When the trade is executed
    trade_time: u64,
    // When the websocket event is received
    event_time: u64,
    // When the feature is processed
    processing_time: u64,

    pub tick_imbalance_bar: TickImbalanceBar,
    pub volume_imbalance_bar: VolumeImbalanceBar,
    pub dollar_imbalance_bar: DollarImbalanceBar,

    // Whether the genesis tick imbalance bar has been created
    pub tick_imbalance_bar_init: bool,
    // Whether the genesis volume imbalance bar has been created
    pub volume_imbalance_bar_init: bool,
    // Whether the genesis dollar imbalance bar has been created
    pub dollar_imbalance_bar_init: bool,
}

impl FeatureInProgress {
    pub fn new() -> Self {
        FeatureInProgress {
            // Time
            trade_time: 0,
            event_time: 0,
            processing_time: 0,

            // Information bars
            tick_imbalance_bar: TickImbalanceBar::new(), // Initialize single tick imbalance bar
            tick_imbalance_bar_init: false,

            volume_imbalance_bar: VolumeImbalanceBar::new(),
            volume_imbalance_bar_init: false,

            dollar_imbalance_bar: DollarImbalanceBar::new(),
            dollar_imbalance_bar_init: false,
        }
    }
}

pub struct StreamBase<Rx> {
    // Receive data from
    pub rx_orderbook: Receiver<OrderbookData>,
    pub rx_market: Receiver<MarketData>,

    // Send data to Executor
    pub tx_feature: Sender<FeatureProcessed>,

    // During the bar creating process, always reset the feature in progress after bar creation
    // Feature processed should not be reset

    // Feature in progress
    pub in_progress: FeatureInProgress,
    // Feature processed
    pub processed: FeatureProcessed,

    pub additional_rx: Rx,
}

pub struct SpotReceivers;
pub struct FutureReceivers {
    pub rx_markprice: Receiver<MarkPriceData>,
    pub rx_liquidation: Receiver<LiquidationData>,
}

impl<Rx> StreamBase<Rx> {
    pub fn update_dollar_imbalance_bar(&mut self, mkt_data: &MarketData) {
        // Update VWAP and imbalance values regardless of bar completion
        self.processed.dollar_imbalance_vwap = self.in_progress.dollar_imbalance_bar.bar.vwap;
        self.processed.dollar_imbalance = self.in_progress.dollar_imbalance_bar.bar.imb;
        self.processed.dollar_imbalance_thres = self.in_progress.dollar_imbalance_bar.bar.imb_thres;
        self.processed.dollar_imbalance_cvd = self.in_progress.dollar_imbalance_bar.cvd;

        if self.in_progress.dollar_imbalance_bar_init {
            if let Some(db) = self.in_progress.dollar_imbalance_bar.bar(mkt_data) {
                self.processed.dollar_imbalance_bar = db.clone();

                // Reset the feature in progress
                log::debug!("New Dollar Imbalance Bar Created");
                self.in_progress.dollar_imbalance_bar.reset();
            }
        } else if let Some(db) = self.in_progress.dollar_imbalance_bar.genesis_bar(mkt_data) {
            self.processed.dollar_imbalance_bar = db.clone();

            // Mark that the first bar has been created
            self.processed.dollar_imbalance_bar_init = true;
            self.in_progress.dollar_imbalance_bar_init = true;

            // Reset the feature in progress
            log::debug!("First Dollar Imbalance Bar Created");
            self.in_progress.dollar_imbalance_bar.reset();
        }
    }

    pub fn update_volume_imbalance_bar(&mut self, mkt_data: &MarketData) {
        // Update VWAP and imbalance values regardless of bar completion
        self.processed.volume_imbalance_vwap = self.in_progress.volume_imbalance_bar.bar.vwap;
        self.processed.volume_imbalance = self.in_progress.volume_imbalance_bar.bar.imb;
        self.processed.volume_imbalance_thres = self.in_progress.volume_imbalance_bar.bar.imb_thres;
        self.processed.volume_imbalance_cvd = self.in_progress.volume_imbalance_bar.cvd;

        if self.in_progress.volume_imbalance_bar_init {
            if let Some(vb) = self.in_progress.volume_imbalance_bar.bar(mkt_data) {
                self.processed.volume_imbalance_bar = vb.clone();

                // Reset the feature in progress
                log::debug!("New Volume Imbalance Bar Created");
                self.in_progress.volume_imbalance_bar.reset();
            }
        } else if let Some(vb) = self.in_progress.volume_imbalance_bar.genesis_bar(mkt_data) {
            self.processed.volume_imbalance_bar = vb.clone();

            // Mark that the first bar has been created
            self.processed.volume_imbalance_bar_init = true;
            self.in_progress.volume_imbalance_bar_init = true;

            // Reset the feature in progress
            log::debug!("First Volume Imbalance Bar Created");
            self.in_progress.volume_imbalance_bar.reset();
        }
    }

    pub fn update_tick_imbalance_bar(&mut self, mkt_data: &MarketData) {
        // Update VWAP and imbalance values regardless of bar completion
        self.processed.tick_imbalance_vwap = self.in_progress.tick_imbalance_bar.bar.vwap;
        self.processed.tick_imbalance = self.in_progress.tick_imbalance_bar.bar.imb;
        self.processed.tick_imbalance_thres = self.in_progress.tick_imbalance_bar.bar.imb_thres;

        if self.in_progress.tick_imbalance_bar_init {
            if let Some(tb) = self.in_progress.tick_imbalance_bar.bar(mkt_data) {
                self.processed.tick_imbalance_bar = tb;

                // Reset the feature in progress
                log::debug!("New Tick Imbalance Bar Created");
                self.in_progress.tick_imbalance_bar.bar.reset();
            }
        } else if let Some(tb) = self.in_progress.tick_imbalance_bar.genesis_bar(mkt_data) {
            self.processed.tick_imbalance_bar = tb;

            // Mark that the first bar has been created
            self.processed.tick_imbalance_bar_init = true;
            self.in_progress.tick_imbalance_bar_init = true;

            // Reset the feature in progress
            log::debug!("First Tick Imbalance Bar Created");
            self.in_progress.tick_imbalance_bar.reset();
        }
    }
}
