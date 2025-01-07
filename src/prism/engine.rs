use tokio::sync::mpsc::{Receiver, Sender};

use crate::data::market::binance_aggtrade_future::MarketData;
use crate::data::orderbook::book::OrderbookData;

pub struct PrismaEngine {
    rx_orderbook: Receiver<OrderbookData>,
    rx_market: Receiver<MarketData>,
    tx_feature: Sender<PrismaFeature>,
    price: Option<f32>,

    latest_update_time: u64,

    // Trade indicators
    feature: PrismaFeature,
}

#[derive(Debug, Clone)]
pub struct PrismaFeature {
    pub time: u64,
    pub source: String,

    pub price: f32,
    pub aggressive_measure_begin: u64,
    pub aggressive_measure_next: u64,

    pub maker_quantity: f32,
    pub taker_quantity: f32,
    pub aggressiveness: f32,
    pub obi: f32,
    pub ofi: f32,
    pub obi_range: (f32, f32, f32, f32),
}

#[allow(dead_code)]
impl PrismaEngine {
    pub fn new(
        source: &str,
        rx_orderbook: Receiver<OrderbookData>,
        rx_market: Receiver<MarketData>,
        tx_feature: Sender<PrismaFeature>,
    ) -> Self {
        // rx_orderbook, rx_market: Collects data from the orderbook and market stream
        // tx_feature: Sends the engineered features to the database / trading executor
        Self {
            rx_orderbook,
            rx_market,
            tx_feature,

            price: None,

            latest_update_time: 0,

            feature: PrismaFeature {
                time: 0,
                source: source.to_string(),
                price: 0.0,
                aggressive_measure_begin: 0,
                aggressive_measure_next: 0,
                maker_quantity: 0f32,
                taker_quantity: 0f32,
                aggressiveness: 0.0,
                obi: 0.0,
                ofi: 0.0,
                obi_range: (0.0, 0.0, 0.0, 0.0),
            },
        }
    }

    pub async fn work(&mut self) {
        loop {
            tokio::select! {
                Some(fut_mkt_data) = self.rx_market.recv() => {
                    self.price = Some(fut_mkt_data.price);
                    self.feature.price = fut_mkt_data.price;
                    self.feature.time = fut_mkt_data.time;

                    match fut_mkt_data.buyer_market_maker {
                        true => self.feature.maker_quantity += fut_mkt_data.quantity,
                        false => self.feature.taker_quantity += fut_mkt_data.quantity,
                    }

                    if self.feature.aggressive_measure_begin == 0 {
                        // Initilize the aggressive measure
                        self.feature.aggressive_measure_begin = fut_mkt_data.time;
                        self.feature.aggressive_measure_next = fut_mkt_data.time + 500; // Add 5 seconds (5000ms)
                    } else if fut_mkt_data.time > self.feature.aggressive_measure_next {
                        self.feature.aggressiveness = self.feature.maker_quantity / (self.feature.taker_quantity + self.feature.maker_quantity);

                        self.feature.aggressive_measure_begin = fut_mkt_data.time;
                        self.feature.aggressive_measure_next = fut_mkt_data.time + 500; // Add 5 seconds (5000ms)

                        // Re initialize the feature.maker_quantity and feature.taker_quantity
                        self.feature.maker_quantity = 0f32;
                        self.feature.taker_quantity = 0f32;
                    }

                    // Send the feature to the database
                    self.tx_feature.send(self.feature.clone()).await.unwrap();
                    self.latest_update_time = fut_mkt_data.time;
                }
                Some(mut fut_ob_data) = self.rx_orderbook.recv() => {
                    if let Some(price) = self.price {
                        self.feature.time = fut_ob_data.time;

                        self.feature.ofi = fut_ob_data.orderflow_imbalance();
                        self.feature.obi = fut_ob_data.orderbook_imbalance();
                        fut_ob_data.update_best_bid_ask(); // Update after calculating flow imbalance

                        self.feature.obi_range.0 = fut_ob_data.orderbook_imbalance_slack(price, 0.01);
                        self.feature.obi_range.1 = fut_ob_data.orderbook_imbalance_slack(price, 0.02);
                        self.feature.obi_range.2 = fut_ob_data.orderbook_imbalance_slack(price, 0.05);
                        self.feature.obi_range.3 = fut_ob_data.orderbook_imbalance_slack(price, 0.10);
                        self.feature.price = price;

                        self.tx_feature.send(self.feature.clone()).await.unwrap();

                        self.latest_update_time = fut_ob_data.time;
                    }
                }
            }
        }
    }
}
