use core::f32;
use log::error;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::data::market::binance_aggtrade_future::MarketData;
use crate::data::orderbook::book::OrderbookData;
use crate::prism::bar_tick_imbalance::TickImbalanceBar;
use crate::prism::bar_volume_imbalance::{VolumeImbalanceBar, VolumeType};

pub struct PrismFeatureEngine {
    // Receive data from
    rx_orderbook: Receiver<OrderbookData>,
    rx_market: Receiver<MarketData>,

    // Send data to Executor
    tx_feature: Sender<PrismaFeature>,

    // Feature + Feature Temporary
    feature: PrismaFeature,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PrismaFeature {
    pub feature_time: u64,
    pub source: String,

    pub price: f32,
    pub maker_quantity: f32,
    pub taker_quantity: f32,

    // Time Bar
    pub obi: f32,                        // Orderbook imbalance
    pub obi_range: (f32, f32, f32, f32), // Ranged Orderbook imbalance

    pub tick_imbalance_bar: TickImbalanceBar,
    tick_imbalance_bar_init: bool,

    pub volume_imbalance_bar_both: VolumeImbalanceBar,
    volume_imbalance_bar_both_init: bool,

    pub volume_imbalance_bar_maker: VolumeImbalanceBar,
    volume_imbalance_bar_maker_init: bool,

    pub volume_imbalance_bar_taker: VolumeImbalanceBar,
    volume_imbalance_bar_taker_init: bool,
}

#[derive(Debug, Clone)]
pub enum PrismaSource {
    Future,
    Spot,
}

impl PrismaSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            PrismaSource::Future => "f",
            PrismaSource::Spot => "s",
        }
    }
}

#[allow(dead_code)]
impl PrismFeatureEngine {
    pub fn new(
        source: PrismaSource,
        rx_orderbook: Receiver<OrderbookData>,
        rx_market: Receiver<MarketData>,
        tx_feature: Sender<PrismaFeature>,
    ) -> Self {
        // rx_orderbook, rx_market: Collects data from the orderbook and market stream
        // tx_feature: Sends the engineered features to the database / trading executor
        const TICK_IMBALANCE_THRESHOLD: f32 = 10.0;

        Self {
            // Attach channels
            rx_orderbook,
            rx_market,
            tx_feature,

            feature: PrismaFeature {
                feature_time: u64::MAX,
                source: source.as_str().to_string(),
                price: -f32::INFINITY,
                maker_quantity: 0.0,
                taker_quantity: 0.0,
                obi: -f32::INFINITY, // Should be inside -1 < obi < 1
                obi_range: (0.0, 0.0, 0.0, 0.0),

                // Information bars
                tick_imbalance_bar: TickImbalanceBar::new(), // Initialize single tick imbalance bar
                tick_imbalance_bar_init: false,

                volume_imbalance_bar_both: VolumeImbalanceBar::new(VolumeType::Both),
                volume_imbalance_bar_both_init: false,

                volume_imbalance_bar_maker: VolumeImbalanceBar::new(VolumeType::Maker),
                volume_imbalance_bar_maker_init: false,

                volume_imbalance_bar_taker: VolumeImbalanceBar::new(VolumeType::Taker),
                volume_imbalance_bar_taker_init: false,
            },
        }
    }

    fn update_volume_imbalance_bar_both(&mut self, mkt_data: &MarketData) {
        if self.feature.volume_imbalance_bar_both_init {
            if let Some(vb) = self.feature.volume_imbalance_bar_both.bar(mkt_data) {
                self.feature.volume_imbalance_bar_both = vb;
            }
        } else if let Some(vb) = self.feature.volume_imbalance_bar_both.genesis_bar(mkt_data) {
            self.feature.volume_imbalance_bar_both = vb;
            self.feature.volume_imbalance_bar_both_init = true;
        }
    }

    fn update_volume_imbalance_bar_maker(&mut self, mkt_data: &MarketData) {
        if self.feature.volume_imbalance_bar_maker_init {
            if let Some(vb) = self.feature.volume_imbalance_bar_maker.bar(mkt_data) {
                self.feature.volume_imbalance_bar_maker = vb;
            }
        } else if let Some(vb) = self
            .feature
            .volume_imbalance_bar_maker
            .genesis_bar(mkt_data)
        {
            self.feature.volume_imbalance_bar_maker = vb;
            self.feature.volume_imbalance_bar_maker_init = true;
        }
    }

    fn update_volume_imbalance_bar_taker(&mut self, mkt_data: &MarketData) {
        if self.feature.volume_imbalance_bar_taker_init {
            if let Some(vb) = self.feature.volume_imbalance_bar_taker.bar(mkt_data) {
                self.feature.volume_imbalance_bar_taker = vb;
            }
        } else if let Some(vb) = self
            .feature
            .volume_imbalance_bar_taker
            .genesis_bar(mkt_data)
        {
            self.feature.volume_imbalance_bar_taker = vb;
            self.feature.volume_imbalance_bar_taker_init = true;
        }
    }

    fn update_tick_imbalance_bar(&mut self, mkt_data: &MarketData) {
        if self.feature.tick_imbalance_bar_init {
            if let Some(tb) = self.feature.tick_imbalance_bar.bar(mkt_data) {
                self.feature.tick_imbalance_bar = tb;
            }
        } else if let Some(tb) = self.feature.tick_imbalance_bar.genesis_bar(mkt_data) {
            self.feature.tick_imbalance_bar = tb;
            self.feature.tick_imbalance_bar_init = true;
        }
    }

    pub async fn work(&mut self) {
        loop {
            tokio::select! {
                Some(fut_mkt_data) = self.rx_market.recv() => {
                    // Update price
                    self.feature.price = fut_mkt_data.price;

                    // Update maker/taker quantity
                    match fut_mkt_data.buyer_market_maker {
                        true => self.feature.maker_quantity += fut_mkt_data.quantity,
                        false => self.feature.taker_quantity += fut_mkt_data.quantity,
                    }

                    // Update bars
                    self.update_volume_imbalance_bar_both(&fut_mkt_data);
                    self.update_volume_imbalance_bar_maker(&fut_mkt_data);
                    self.update_volume_imbalance_bar_taker(&fut_mkt_data);
                    self.update_tick_imbalance_bar(&fut_mkt_data);

                    // Update feature time
                    self.feature.feature_time = fut_mkt_data.time;

                    // Send feature to executor - Send every tick
                    if self.tx_feature.send(self.feature.clone()).await.is_err() {
                        error!("Failed to send feature to executor");
                    }

                    // Reset maker/taker quantity
                    self.feature.maker_quantity = 0.0;
                    self.feature.taker_quantity = 0.0;
                }
                Some(mut fut_ob_data) = self.rx_orderbook.recv() => {
                    if self.feature.price > 0.0 {
                        self.feature.feature_time = fut_ob_data.time;

                        self.feature.obi = fut_ob_data.orderbook_imbalance();
                        fut_ob_data.update_best_bid_ask(); // Update after calculating flow imbalance

                        self.feature.obi_range.0 = fut_ob_data.orderbook_imbalance_slack(self.feature.price, 0.005);
                        self.feature.obi_range.1 = fut_ob_data.orderbook_imbalance_slack(self.feature.price, 0.01);
                        self.feature.obi_range.2 = fut_ob_data.orderbook_imbalance_slack(self.feature.price, 0.02);
                        self.feature.obi_range.3 = fut_ob_data.orderbook_imbalance_slack(self.feature.price, 0.5);

                        if self.tx_feature.send(self.feature.clone()).await.is_err() {
                            error!("Failed to send feature to executor");
                        }
                    }
                }
            }
        }
    }
}
