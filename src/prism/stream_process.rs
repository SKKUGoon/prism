use crate::data::{
    liquidation::binance_liquidation_future::LiquidationData,
    market::binance_aggtrade_future::MarketData,
    markprice::binance_markprice_future::MarkPriceData, orderbook::book::OrderbookData,
};
use crate::prism::bar::{DollarImbalanceBar, TickImbalanceBar, VolumeImbalanceBar};
use crate::prism::AssetSource;
use core::f32;
use log::error;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc::{Receiver, Sender};

pub struct StreamProcessor {
    // Receive data from
    rx_orderbook: Receiver<OrderbookData>,
    rx_market: Receiver<MarketData>,
    rx_markprice: Receiver<MarkPriceData>,
    rx_liquidation: Receiver<LiquidationData>,

    // Send data to Executor
    tx_feature: Sender<FeatureProcessed>,

    // During the bar creating process, always reset the feature in progress after bar creation
    // Feature processed should not be reset

    // Feature in progress
    fip: FeatureInProgress,
    // Feature processed
    fpd: FeatureProcessed,
}

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

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct FeatureInProgress {
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

#[allow(dead_code)]
impl StreamProcessor {
    pub fn new(
        source: AssetSource,
        rx_orderbook: Receiver<OrderbookData>,
        rx_market: Receiver<MarketData>,
        rx_markprice: Receiver<MarkPriceData>,
        rx_liquidation: Receiver<LiquidationData>,
        tx_feature: Sender<FeatureProcessed>,
    ) -> Self {
        // rx_orderbook, rx_market: Collects data from the orderbook and market stream
        // tx_feature: Sends the engineered features to the database / trading executor

        Self {
            // Attach channels
            rx_orderbook,
            rx_market,
            rx_markprice,
            rx_liquidation,
            tx_feature,

            fip: FeatureInProgress {
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
            },

            fpd: FeatureProcessed {
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
            },
        }
    }

    fn update_dollar_imbalance_bar(&mut self, mkt_data: &MarketData) {
        // Update VWAP and imbalance values regardless of bar completion
        self.fpd.dollar_imbalance_vwap = self.fip.dollar_imbalance_bar.bar.vwap;
        self.fpd.dollar_imbalance = self.fip.dollar_imbalance_bar.bar.imb;
        self.fpd.dollar_imbalance_thres = self.fip.dollar_imbalance_bar.bar.imb_thres;
        self.fpd.dollar_imbalance_cvd = self.fip.dollar_imbalance_bar.cvd;

        if self.fip.dollar_imbalance_bar_init {
            if let Some(db) = self.fip.dollar_imbalance_bar.bar(mkt_data) {
                self.fpd.dollar_imbalance_bar = db.clone();

                // Reset the feature in progress
                log::debug!("New Dollar Imbalance Bar Created");
                self.fip.dollar_imbalance_bar.reset();
            }
        } else if let Some(db) = self.fip.dollar_imbalance_bar.genesis_bar(mkt_data) {
            self.fpd.dollar_imbalance_bar = db.clone();

            // Mark that the first bar has been created
            self.fpd.dollar_imbalance_bar_init = true;
            self.fip.dollar_imbalance_bar_init = true;

            // Reset the feature in progress
            log::debug!("First Dollar Imbalance Bar Created");
            self.fip.dollar_imbalance_bar.reset();
        }
    }

    fn update_volume_imbalance_bar(&mut self, mkt_data: &MarketData) {
        // Update VWAP and imbalance values regardless of bar completion
        self.fpd.volume_imbalance_vwap = self.fip.volume_imbalance_bar.bar.vwap;
        self.fpd.volume_imbalance = self.fip.volume_imbalance_bar.bar.imb;
        self.fpd.volume_imbalance_thres = self.fip.volume_imbalance_bar.bar.imb_thres;
        self.fpd.volume_imbalance_cvd = self.fip.volume_imbalance_bar.cvd;

        if self.fip.volume_imbalance_bar_init {
            if let Some(vb) = self.fip.volume_imbalance_bar.bar(mkt_data) {
                self.fpd.volume_imbalance_bar = vb.clone();

                // Reset the feature in progress
                log::debug!("New Volume Imbalance Bar Created");
                self.fip.volume_imbalance_bar.reset();
            }
        } else if let Some(vb) = self.fip.volume_imbalance_bar.genesis_bar(mkt_data) {
            self.fpd.volume_imbalance_bar = vb.clone();

            // Mark that the first bar has been created
            self.fpd.volume_imbalance_bar_init = true;
            self.fip.volume_imbalance_bar_init = true;

            // Reset the feature in progress
            log::debug!("First Volume Imbalance Bar Created");
            self.fip.volume_imbalance_bar.reset();
        }
    }

    fn update_tick_imbalance_bar(&mut self, mkt_data: &MarketData) {
        // Update VWAP and imbalance values regardless of bar completion
        self.fpd.tick_imbalance_vwap = self.fip.tick_imbalance_bar.bar.vwap;
        self.fpd.tick_imbalance = self.fip.tick_imbalance_bar.bar.imb;
        self.fpd.tick_imbalance_thres = self.fip.tick_imbalance_bar.bar.imb_thres;

        if self.fip.tick_imbalance_bar_init {
            if let Some(tb) = self.fip.tick_imbalance_bar.bar(mkt_data) {
                self.fpd.tick_imbalance_bar = tb.clone();

                // Reset the feature in progress
                log::debug!("New Tick Imbalance Bar Created");
                self.fip.tick_imbalance_bar.bar.reset();
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
                    self.update_volume_imbalance_bar(&fut_mkt_data);
                    self.update_tick_imbalance_bar(&fut_mkt_data);
                    self.update_dollar_imbalance_bar(&fut_mkt_data);

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

                // Insert Mark Price Data
                Some(fut_markprice_data) = self.rx_markprice.recv() => {
                    self.fpd.event_type = Some("Mark Price".to_string());
                    self.fpd.event_time = fut_markprice_data.event_time;
                    self.fpd.processed_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;

                    self.fpd.mark_price = fut_markprice_data.mark_price;
                    self.fpd.funding_rate = fut_markprice_data.funding_rate;
                    self.fpd.next_funding_time = fut_markprice_data.next_funding_time;

                    if self.tx_feature.send(self.fpd.clone()).await.is_err() {
                        error!("Failed to send feature to executor");
                    }

                    // Re-Initialize mark price data
                    self.fpd.mark_price = 0.0;
                    self.fpd.funding_rate = 0.0;
                }

                // Insert Liquidation Data
                Some(fut_liquidation_data) = self.rx_liquidation.recv() => {
                    // Update price
                    self.fpd.event_type = Some("Liquidation".to_string());
                    self.fpd.event_time = fut_liquidation_data.event_time;
                    self.fpd.processed_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;

                    self.fpd.liquidation_side = fut_liquidation_data.side;
                    self.fpd.liquidation_dvolume = fut_liquidation_data.quantity * fut_liquidation_data.avg_price;

                    if self.tx_feature.send(self.fpd.clone()).await.is_err() {
                        error!("Failed to send feature to executor");
                    }

                    // Re-Initialize liquidation data
                    self.fpd.liquidation_side = "".to_string();
                    self.fpd.liquidation_dvolume = 0.0;
                }
            }
        }
    }
}
