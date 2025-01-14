use core::f32;
use log::error;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::data::market::binance_aggtrade_future::MarketData;
use crate::data::orderbook::book::OrderbookData;
use crate::prism::bar_tick_imbalance::Tib;

pub struct PrismFeatureEngine {
    // Receive data from
    rx_orderbook: Receiver<OrderbookData>,
    rx_market: Receiver<MarketData>,

    // Send data to Executor
    tx_feature: Sender<PrismaFeature>,

    // Feature + Feature Temporary
    feature: PrismaFeature,
    temporary: PrismaFeature,
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

    // Tick Imbalance Bar
    // Bars will be generated from the `temporary` attributes.
    // When bars are generated, they will be moved to the `feature` attributes.
    pub tib: Tib,
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
                tib: Tib::new(TICK_IMBALANCE_THRESHOLD), // Initialize single tick imbalance bar
                obi: -f32::INFINITY,                     // Should be inside -1 < obi < 1
                obi_range: (0.0, 0.0, 0.0, 0.0),
            },
            temporary: PrismaFeature {
                feature_time: u64::MAX,              // Not used in temporary
                source: source.as_str().to_string(), // Not used in temporary
                price: -f32::INFINITY,               // Not used in temporary
                maker_quantity: 0.0,                 // Not used in temporary
                taker_quantity: 0.0,                 // Not used in temporary
                tib: Tib::new(TICK_IMBALANCE_THRESHOLD),
                obi: -f32::INFINITY,             // Not used in temporary
                obi_range: (0.0, 0.0, 0.0, 0.0), // Not used in temporary
            },
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

                    if let Some(tb) = self.temporary.tib.update(&fut_mkt_data) {
                        // Update tick imbalance bar
                        self.feature.tib = tb;
                        self.temporary.tib.reset();
                    }

                    // Update feature time
                    self.feature.feature_time = fut_mkt_data.time;

                    // Send feature to executor
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
