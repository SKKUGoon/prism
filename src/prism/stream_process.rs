use crate::data::market::binance_aggtrade_future::MarketData;
use crate::data::orderbook::book::OrderbookData;
use crate::prism::bar_dollar_imbalance::{DollarImbalanceBar, DollarVolumeType};
use crate::prism::bar_tick_imbalance::TickImbalanceBar;
use crate::prism::bar_volume_imbalance::{VolumeImbalanceBar, VolumeType};
use crate::prism::AssetSource;
use core::f32;
use log::error;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc::{Receiver, Sender};

pub struct StreamProcessor {
    // Receive data from
    rx_orderbook: Receiver<OrderbookData>,
    rx_market: Receiver<MarketData>,

    // Send data to Executor
    tx_feature: Sender<FeatureProcessed>,

    // Feature + Feature Temporary
    // During the bar creating process, always reset the feature in progress after bar creation
    // Feature processed should not be reset
    fip: FeatureInProgress, // Feature in progress
    fpd: FeatureProcessed,  // Feature processed
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FeatureProcessed {
    pub event_time: u64,
    pub processed_time: u64,
    pub trade_time: u64,

    pub source: String,

    pub price: f32,
    pub maker_quantity: f32,
    pub taker_quantity: f32,

    pub ob_spread: f32,        // Best ask - best bid
    pub obi: f32,              // Orderbook imbalance
    pub obi_range: (f32, f32), // Ranged Orderbook imbalance

    // Processed Bars (old one for the queue)
    pub tick_imbalance_bar: TickImbalanceBar,
    pub volume_imbalance_bar_both: VolumeImbalanceBar,
    pub volume_imbalance_bar_maker: VolumeImbalanceBar,
    pub volume_imbalance_bar_taker: VolumeImbalanceBar,
    pub dollar_imbalance_bar_both: DollarImbalanceBar,

    // Real time information
    pub tick_imbalance: f32,
    pub volume_imbalance_both: f32,
    pub volume_imbalance_maker: f32,
    pub volume_imbalance_taker: f32,
    pub dollar_imbalance: f32,

    pub tick_imbalance_thres: f32,
    pub volume_imbalance_both_thres: f32,
    pub volume_imbalance_maker_thres: f32,
    pub volume_imbalance_taker_thres: f32,
    pub dollar_imbalance_thres: f32,

    pub tick_imbalance_vwap: f32,
    pub volume_imbalance_both_vwap: f32,
    pub volume_imbalance_maker_vwap: f32,
    pub volume_imbalance_taker_vwap: f32,
    pub dollar_imbalance_both_vwap: f32,

    tick_imbalance_bar_init: bool,
    volume_imbalance_bar_both_init: bool,
    volume_imbalance_bar_maker_init: bool,
    volume_imbalance_bar_taker_init: bool,
    dollar_imbalance_bar_both_init: bool,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct FeatureInProgress {
    // Time
    trade_time: u64,
    event_time: u64,
    processing_time: u64,

    pub tick_imbalance_bar: TickImbalanceBar,
    pub volume_imbalance_bar_both: VolumeImbalanceBar,
    pub volume_imbalance_bar_maker: VolumeImbalanceBar,
    pub volume_imbalance_bar_taker: VolumeImbalanceBar,
    pub dollar_imbalance_bar_both: DollarImbalanceBar,

    tick_imbalance_bar_init: bool,
    volume_imbalance_bar_both_init: bool,
    volume_imbalance_bar_maker_init: bool,
    volume_imbalance_bar_taker_init: bool,
    dollar_imbalance_bar_both_init: bool,
}

#[allow(dead_code)]
impl StreamProcessor {
    pub fn new(
        source: AssetSource,
        rx_orderbook: Receiver<OrderbookData>,
        rx_market: Receiver<MarketData>,
        tx_feature: Sender<FeatureProcessed>,
    ) -> Self {
        // rx_orderbook, rx_market: Collects data from the orderbook and market stream
        // tx_feature: Sends the engineered features to the database / trading executor

        Self {
            // Attach channels
            rx_orderbook,
            rx_market,
            tx_feature,

            fip: FeatureInProgress {
                // Time
                trade_time: 0,
                event_time: 0,
                processing_time: 0,

                // Information bars
                tick_imbalance_bar: TickImbalanceBar::new(), // Initialize single tick imbalance bar
                tick_imbalance_bar_init: false,

                volume_imbalance_bar_both: VolumeImbalanceBar::new(VolumeType::Both),
                volume_imbalance_bar_both_init: false,

                volume_imbalance_bar_maker: VolumeImbalanceBar::new(VolumeType::Maker),
                volume_imbalance_bar_maker_init: false,

                volume_imbalance_bar_taker: VolumeImbalanceBar::new(VolumeType::Taker),
                volume_imbalance_bar_taker_init: false,

                dollar_imbalance_bar_both: DollarImbalanceBar::new(DollarVolumeType::Both),
                dollar_imbalance_bar_both_init: false,
            },

            fpd: FeatureProcessed {
                // Time
                trade_time: 0,
                event_time: 0,
                processed_time: 0,

                source: source.as_str().to_string(),
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

                volume_imbalance_bar_both: VolumeImbalanceBar::new(VolumeType::Both),
                volume_imbalance_bar_both_init: false,
                volume_imbalance_both: 0.0,
                volume_imbalance_both_vwap: 0.0,
                volume_imbalance_both_thres: 0.0,

                volume_imbalance_bar_maker: VolumeImbalanceBar::new(VolumeType::Maker),
                volume_imbalance_bar_maker_init: false,
                volume_imbalance_maker: 0.0,
                volume_imbalance_maker_vwap: 0.0,
                volume_imbalance_maker_thres: 0.0,

                volume_imbalance_bar_taker: VolumeImbalanceBar::new(VolumeType::Taker),
                volume_imbalance_bar_taker_init: false,
                volume_imbalance_taker: 0.0,
                volume_imbalance_taker_vwap: 0.0,
                volume_imbalance_taker_thres: 0.0,

                dollar_imbalance_bar_both: DollarImbalanceBar::new(DollarVolumeType::Both),
                dollar_imbalance_bar_both_init: false,
                dollar_imbalance: 0.0,
                dollar_imbalance_both_vwap: 0.0,
                dollar_imbalance_thres: 0.0,
            },
        }
    }

    fn update_dollar_imbalance_bar_both(&mut self, mkt_data: &MarketData) {
        // Update VWAP and imbalance values regardless of bar completion
        self.fpd.dollar_imbalance_both_vwap = self.fip.dollar_imbalance_bar_both.vwap;
        self.fpd.dollar_imbalance = self.fip.dollar_imbalance_bar_both.imb;
        self.fpd.dollar_imbalance_thres = self.fip.dollar_imbalance_bar_both.imb_thres;

        if self.fip.dollar_imbalance_bar_both_init {
            if let Some(db) = self.fip.dollar_imbalance_bar_both.bar(mkt_data) {
                self.fpd.dollar_imbalance_bar_both = db.clone();

                // Reset the feature in progress
                log::debug!("New Dollar Imbalance Bar Created");
                self.fip.dollar_imbalance_bar_both.reset();
            }
        } else if let Some(db) = self.fip.dollar_imbalance_bar_both.genesis_bar(mkt_data) {
            self.fpd.dollar_imbalance_bar_both = db.clone();

            // Mark that the first bar has been created
            self.fpd.dollar_imbalance_bar_both_init = true;
            self.fip.dollar_imbalance_bar_both_init = true;

            // Reset the feature in progress
            log::debug!("First Dollar Imbalance Bar Created");
            self.fip.dollar_imbalance_bar_both.reset();
        }
    }

    fn update_volume_imbalance_bar_both(&mut self, mkt_data: &MarketData) {
        // Update VWAP and imbalance values regardless of bar completion
        self.fpd.volume_imbalance_both_vwap = self.fip.volume_imbalance_bar_both.vwap;
        self.fpd.volume_imbalance_both = self.fip.volume_imbalance_bar_both.imb;
        self.fpd.volume_imbalance_both_thres = self.fip.volume_imbalance_bar_both.imb_thres;

        if self.fip.volume_imbalance_bar_both_init {
            if let Some(vb) = self.fip.volume_imbalance_bar_both.bar(mkt_data) {
                self.fpd.volume_imbalance_bar_both = vb.clone();

                // Reset the feature in progress
                log::debug!("New Volume Imbalance Bar Created");
                self.fip.volume_imbalance_bar_both.reset();
            }
        } else if let Some(vb) = self.fip.volume_imbalance_bar_both.genesis_bar(mkt_data) {
            self.fpd.volume_imbalance_bar_both = vb.clone();

            // Mark that the first bar has been created
            self.fpd.volume_imbalance_bar_both_init = true;
            self.fip.volume_imbalance_bar_both_init = true;

            // Reset the feature in progress
            log::debug!("First Volume Imbalance Bar Created");
            self.fip.volume_imbalance_bar_both.reset();
        }
    }

    fn update_volume_imbalance_bar_maker(&mut self, mkt_data: &MarketData) {
        // Update VWAP and imbalance values regardless of bar completion
        self.fpd.volume_imbalance_maker_vwap = self.fip.volume_imbalance_bar_maker.vwap;
        self.fpd.volume_imbalance_maker = self.fip.volume_imbalance_bar_maker.imb;
        self.fpd.volume_imbalance_maker_thres = self.fip.volume_imbalance_bar_maker.imb_thres;

        if self.fip.volume_imbalance_bar_maker_init {
            if let Some(vb) = self.fip.volume_imbalance_bar_maker.bar(mkt_data) {
                self.fpd.volume_imbalance_bar_maker = vb.clone();

                // Reset the feature in progress
                log::debug!("New Volume Imbalance Bar Created");
                self.fip.volume_imbalance_bar_maker.reset();
            }
        } else if let Some(vb) = self.fip.volume_imbalance_bar_maker.genesis_bar(mkt_data) {
            self.fpd.volume_imbalance_bar_maker = vb.clone();

            // Mark that the first bar has been created
            self.fpd.volume_imbalance_bar_maker_init = true;
            self.fip.volume_imbalance_bar_maker_init = true;

            // Reset the feature in progress
            log::debug!("First Volume Imbalance Bar Created");
            self.fip.volume_imbalance_bar_maker.reset();
        }
    }

    fn update_volume_imbalance_bar_taker(&mut self, mkt_data: &MarketData) {
        // Update VWAP and imbalance values regardless of bar completion
        self.fpd.volume_imbalance_taker_vwap = self.fip.volume_imbalance_bar_taker.vwap;
        self.fpd.volume_imbalance_taker = self.fip.volume_imbalance_bar_taker.imb;
        self.fpd.volume_imbalance_taker_thres = self.fip.volume_imbalance_bar_taker.imb_thres;

        if self.fip.volume_imbalance_bar_taker_init {
            if let Some(vb) = self.fip.volume_imbalance_bar_taker.bar(mkt_data) {
                self.fpd.volume_imbalance_bar_taker = vb.clone();

                // Reset the feature in progress
                log::debug!("New Volume Imbalance Bar Created");
                self.fip.volume_imbalance_bar_taker.reset();
            }
        } else if let Some(vb) = self.fip.volume_imbalance_bar_taker.genesis_bar(mkt_data) {
            self.fpd.volume_imbalance_bar_taker = vb.clone();

            // Mark that the first bar has been created
            self.fpd.volume_imbalance_bar_taker_init = true;
            self.fip.volume_imbalance_bar_taker_init = true;

            // Reset the feature in progress
            log::debug!("First Volume Imbalance Bar Created");
            self.fip.volume_imbalance_bar_taker.reset();
        }
    }

    fn update_tick_imbalance_bar(&mut self, mkt_data: &MarketData) {
        // Update VWAP and imbalance values regardless of bar completion
        self.fpd.tick_imbalance_vwap = self.fip.tick_imbalance_bar.vwap;
        self.fpd.tick_imbalance = self.fip.tick_imbalance_bar.imb;
        self.fpd.tick_imbalance_thres = self.fip.tick_imbalance_bar.imb_thres;

        if self.fip.tick_imbalance_bar_init {
            if let Some(tb) = self.fip.tick_imbalance_bar.bar(mkt_data) {
                self.fpd.tick_imbalance_bar = tb.clone();

                // Reset the feature in progress
                log::debug!("New Tick Imbalance Bar Created");
                self.fip.tick_imbalance_bar.reset();
            }
        } else if let Some(tb) = self.fip.tick_imbalance_bar.genesis_bar(mkt_data) {
            self.fpd.tick_imbalance_bar = tb.clone();

            // Mark that the first bar has been created
            self.fpd.tick_imbalance_bar_init = true;
            self.fip.tick_imbalance_bar_init = true;

            // Reset the feature in progress
            log::debug!("First Tick Imbalance Bar Created");
            self.fip.tick_imbalance_bar.reset();
        }
    }

    pub async fn work(&mut self) {
        loop {
            tokio::select! {
                // Insert Aggregate Trade MarketData to create Bar
                Some(fut_mkt_data) = self.rx_market.recv() => {
                    // Update price
                    self.fpd.price = fut_mkt_data.price;

                    // Update maker/taker quantity
                    match fut_mkt_data.buyer_market_maker {
                        true => self.fpd.maker_quantity += fut_mkt_data.quantity,
                        false => self.fpd.taker_quantity += fut_mkt_data.quantity,
                    }

                    // Update bars
                    self.update_volume_imbalance_bar_both(&fut_mkt_data);
                    self.update_volume_imbalance_bar_maker(&fut_mkt_data);
                    self.update_volume_imbalance_bar_taker(&fut_mkt_data);
                    self.update_tick_imbalance_bar(&fut_mkt_data);
                    self.update_dollar_imbalance_bar_both(&fut_mkt_data);

                    // Update feature time
                    self.fpd.trade_time = fut_mkt_data.trade_time;
                    self.fpd.event_time = fut_mkt_data.event_time;
                    self.fpd.processed_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;

                    if self.tx_feature.send(self.fpd.clone()).await.is_err() {
                        error!("Failed to send feature to executor");
                    }

                    // Reset maker/taker quantity
                    self.fpd.maker_quantity = 0.0;
                    self.fpd.taker_quantity = 0.0;
                }

                // Insert Orderbook Data
                Some(mut fut_ob_data) = self.rx_orderbook.recv() => {
                    if self.fpd.price > 0.0 {
                        self.fpd.trade_time = fut_ob_data.trade_time;
                        self.fpd.event_time = fut_ob_data.event_time;
                        self.fpd.processed_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;

                        self.fpd.obi = fut_ob_data.orderbook_imbalance();
                        fut_ob_data.update_best_bid_ask(); // Update after calculating flow imbalance
                        self.fpd.ob_spread = fut_ob_data.best_ask.0.parse::<f32>().unwrap_or(0.0) - fut_ob_data.best_bid.0.parse::<f32>().unwrap_or(0.0);

                        self.fpd.obi_range.0 = fut_ob_data.orderbook_imbalance_slack(self.fpd.price, 0.005);
                        self.fpd.obi_range.1 = fut_ob_data.orderbook_imbalance_slack(self.fpd.price, 0.01);

                        if self.tx_feature.send(self.fpd.clone()).await.is_err() {
                            error!("Failed to send feature to executor");
                        }
                    }
                }
            }
        }
    }
}
