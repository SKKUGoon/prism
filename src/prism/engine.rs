use log::info;
use tokio::sync::mpsc::Receiver;

use crate::data::market::binance_aggtrade_future::MarketData;
use crate::data::orderbook::book::OrderbookData;

pub struct PrismaEngine {
    rx_future_orderbook: Receiver<OrderbookData>,
    rx_future_market: Receiver<MarketData>,

    rx_spot_orderbook: Receiver<OrderbookData>,
    rx_spot_market: Receiver<MarketData>,

    fut_price: Option<f32>,
    spot_price: Option<f32>,

    latest_update_time: u64,

    // Trade indicators
    fut_feature: PrismaFeature,
    spot_feature: PrismaFeature,

    #[allow(dead_code)]
    trade_config: PrismaTradeConfig, // Not used yet
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct PrismaFeature {
    aggressive_measure_begin: u64,
    aggressive_measure_next: u64,
    maker_quantity: f32,
    taker_quantity: f32,
    aggressiveness: f32,
    obi: (f32, f32),
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct PrismaTradeConfig {
    leverage: u8,
    max_leverage: u8,

    loss_cut: f32,
    take_profit: f32,

    max_position_size: f32,
}

#[allow(dead_code)]
impl PrismaEngine {
    pub fn new(
        rx_future_orderbook: Receiver<OrderbookData>,
        rx_future_market: Receiver<MarketData>,

        rx_spot_orderbook: Receiver<OrderbookData>,
        rx_spot_market: Receiver<MarketData>,
    ) -> Self {
        Self {
            rx_future_orderbook,
            rx_future_market,

            rx_spot_orderbook,
            rx_spot_market,

            fut_price: None,
            spot_price: None,

            latest_update_time: 0,

            fut_feature: PrismaFeature {
                aggressive_measure_begin: 0,
                aggressive_measure_next: 0,
                maker_quantity: 0f32,
                taker_quantity: 0f32,
                aggressiveness: 0.0,
                obi: (0.0, 0.0),
            },

            spot_feature: PrismaFeature {
                aggressive_measure_begin: 0,
                aggressive_measure_next: 0,
                maker_quantity: 0f32,
                taker_quantity: 0f32,
                aggressiveness: 0.0,
                obi: (0.0, 0.0),
            },

            trade_config: PrismaTradeConfig {
                leverage: 1,
                max_leverage: 30,
                loss_cut: 0.5,
                take_profit: 0.10,
                max_position_size: 1000.0,
            },
        }
    }

    pub async fn work(&mut self) {
        loop {
            tokio::select! {
                Some(fut_mkt_data) = self.rx_future_market.recv() => {
                    self.fut_price = Some(fut_mkt_data.price);

                    match fut_mkt_data.buyer_market_maker {
                        true => self.fut_feature.maker_quantity += fut_mkt_data.quantity,
                        false => self.fut_feature.taker_quantity += fut_mkt_data.quantity,
                    }

                    if self.fut_feature.aggressive_measure_begin == 0 {
                        // Initilize the aggressive measure
                        self.fut_feature.aggressive_measure_begin = fut_mkt_data.time;
                        self.fut_feature.aggressive_measure_next = fut_mkt_data.time + 500; // Add 5 seconds (5000ms)
                    } else if fut_mkt_data.time > self.fut_feature.aggressive_measure_next {
                        self.fut_feature.aggressiveness = self.fut_feature.maker_quantity / (self.fut_feature.taker_quantity + self.fut_feature.maker_quantity);

                        self.fut_feature.aggressive_measure_begin = fut_mkt_data.time;
                        self.fut_feature.aggressive_measure_next = fut_mkt_data.time + 500; // Add 5 seconds (5000ms)

                        // Re initialize the feature.maker_quantity and feature.taker_quantity
                        self.fut_feature.maker_quantity = 0f32;
                        self.fut_feature.taker_quantity = 0f32;
                    }

                    self.latest_update_time = fut_mkt_data.time;
                }
                Some(mut fut_ob_data) = self.rx_future_orderbook.recv() => {
                    if let Some(price) = self.fut_price {
                        self.fut_feature.obi.0 = fut_ob_data.orderbook_imbalance(price, 0.05);
                        self.fut_feature.obi.1 = fut_ob_data.orderbook_imbalance(price, 0.10);

                        // TODO: This data can be dumped to a database
                        info!(
                            "Future || Price: {:.4} | OBI 5%: {:.4} | OBI 10%: {:.4} | Aggressiveness: {:.4} | Time: {}",
                            price, self.fut_feature.obi.0, self.fut_feature.obi.1, self.fut_feature.aggressiveness, self.latest_update_time
                        );

                        self.latest_update_time = fut_ob_data.time;
                    }
                }
                Some(spot_mkt_data) = self.rx_spot_market.recv() => {
                    self.spot_price = Some(spot_mkt_data.price);

                    match spot_mkt_data.buyer_market_maker {
                        true => self.spot_feature.maker_quantity += spot_mkt_data.quantity,
                        false => self.spot_feature.taker_quantity += spot_mkt_data.quantity,
                    }

                    if self.spot_feature.aggressive_measure_begin == 0 {
                        // Initilize the aggressive measure
                        self.spot_feature.aggressive_measure_begin = spot_mkt_data.time;
                        self.spot_feature.aggressive_measure_next = spot_mkt_data.time + 500; // Add 5 seconds (5000ms)
                    } else if spot_mkt_data.time > self.spot_feature.aggressive_measure_next {
                        self.spot_feature.aggressiveness = self.spot_feature.maker_quantity / (self.spot_feature.taker_quantity + self.spot_feature.maker_quantity);

                        self.spot_feature.aggressive_measure_begin = spot_mkt_data.time;
                        self.spot_feature.aggressive_measure_next = spot_mkt_data.time + 500; // Add 5 seconds (5000ms)

                        // Re initialize the feature.maker_quantity and feature.taker_quantity
                        self.spot_feature.maker_quantity = 0f32;
                        self.spot_feature.taker_quantity = 0f32;
                    }

                    self.latest_update_time = spot_mkt_data.time;
                }
                Some(mut spot_ob_data) = self.rx_spot_orderbook.recv() => {
                    if let Some(price) = self.spot_price {
                        self.spot_feature.obi.0 = spot_ob_data.orderbook_imbalance(price, 0.05);
                        self.spot_feature.obi.1 = spot_ob_data.orderbook_imbalance(price, 0.10);

                        // TODO: This data can be dumped to a database
                        info!(
                            "Spot   || Price: {:.4} | OBI 5%: {:.4} | OBI 10%: {:.4} | Aggressiveness: {:.4} | Time: {}",
                            price, self.spot_feature.obi.0, self.spot_feature.obi.1, self.spot_feature.aggressiveness, self.latest_update_time
                        );

                        self.latest_update_time = spot_ob_data.time;
                    }
                }
            }
        }
    }
}
