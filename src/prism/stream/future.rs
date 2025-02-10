use crate::data::{market::binance_aggtrade_future::MarketData, orderbook::book::OrderbookData};
use crate::prism::stream::{FeatureInProgress, FeatureProcessed, FutureReceivers, StreamBase};
use log::error;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc::{Receiver, Sender};

pub type FutureStreamProcessor = StreamBase<FutureReceivers>;

impl FutureStreamProcessor {
    pub fn new(
        rx_orderbook: Receiver<OrderbookData>,
        rx_market: Receiver<MarketData>,
        additional_rx: FutureReceivers,
        tx_feature: Sender<FeatureProcessed>,
    ) -> Self {
        StreamBase {
            rx_orderbook,
            rx_market,
            tx_feature,
            fip: FeatureInProgress::new(),
            fpd: FeatureProcessed::new(),
            additional_rx,
        }
    }
}

impl FutureStreamProcessor {
    pub async fn work(&mut self) {
        loop {
            tokio::select! {
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
                Some(fut_markprice_data) = self.additional_rx.rx_markprice.recv() => {
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
                Some(fut_liquidation_data) = self.additional_rx.rx_liquidation.recv() => {
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
