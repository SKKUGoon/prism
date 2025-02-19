use crate::data::{market::MarketData, orderbook::book::OrderbookData};
use crate::prism::stream::{FeatureInProgress, FeatureProcessed, FutureReceivers, StreamBase};
use log::error;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc::{Receiver, Sender};

pub type FutureStream = StreamBase<FutureReceivers>;

impl FutureStream {
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
            in_progress: FeatureInProgress::new(),
            processed: FeatureProcessed::new(),
            additional_rx,
        }
    }

    pub async fn work(&mut self) {
        loop {
            tokio::select! {
                Some(mkt_data) = self.rx_market.recv() => {
                    // Update price
                    self.processed.price = mkt_data.price;

                    // Update event type
                    self.processed.event_type = Some("AggTrade".to_string());

                    // Update maker/taker quantity
                    match mkt_data.buyer_market_maker {
                        true => self.processed.maker_quantity += mkt_data.quantity,
                        false => self.processed.taker_quantity += mkt_data.quantity,
                    }

                    // Update bars
                    self.update_volume_imbalance_bar(&mkt_data);
                    self.update_tick_imbalance_bar(&mkt_data);
                    self.update_dollar_imbalance_bar(&mkt_data);

                    // Update feature time
                    self.processed.trade_time = mkt_data.trade_time;
                    self.processed.event_time = mkt_data.event_time;
                    self.processed.processed_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;

                    if self.tx_feature.send(self.processed.clone()).await.is_err() {
                        error!("Failed to send feature to executor");
                    }

                    // Reset maker/taker quantity
                    self.processed.maker_quantity = 0.0;
                    self.processed.taker_quantity = 0.0;
                }

                // Insert Orderbook Data
                Some(mut ob_data) = self.rx_orderbook.recv() => {
                    if self.processed.price > 0.0 {
                        // Update event type
                        self.processed.event_type = Some("Orderbook".to_string());

                        // Update feature time
                        self.processed.trade_time = ob_data.trade_time;
                        self.processed.event_time = ob_data.event_time;
                        self.processed.processed_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;

                        self.processed.obi = ob_data.orderbook_imbalance();
                        ob_data.update_best_bid_ask(); // Update after calculating flow imbalance
                        self.processed.ob_spread = ob_data.best_ask.0.parse::<f32>().unwrap_or(0.0) - ob_data.best_bid.0.parse::<f32>().unwrap_or(0.0);

                        self.processed.obi_range.0 = ob_data.orderbook_imbalance_slack(self.processed.price, 0.005);
                        self.processed.obi_range.1 = ob_data.orderbook_imbalance_slack(self.processed.price, 0.01);

                        let (bid_activity, ask_activity) = ob_data.near_price_bid_ask_activity(self.processed.price, 0.001);
                        self.processed.near_price_bids = bid_activity;
                        self.processed.near_price_asks = ask_activity;

                        if self.tx_feature.send(self.processed.clone()).await.is_err() {
                            error!("Failed to send feature to executor");
                        }
                    }
                }

                // Insert Mark Price Data
                Some(mprc_data) = self.additional_rx.rx_markprice.recv() => {
                    self.processed.event_type = Some("Mark Price".to_string());
                    self.processed.event_time = mprc_data.event_time;
                    self.processed.processed_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;

                    self.processed.mark_price = mprc_data.mark_price;
                    self.processed.funding_rate = mprc_data.funding_rate;
                    self.processed.next_funding_time = mprc_data.next_funding_time;

                    if self.tx_feature.send(self.processed.clone()).await.is_err() {
                        error!("Failed to send feature to executor");
                    }

                    // Re-Initialize mark price data
                    self.processed.mark_price = 0.0;
                    self.processed.funding_rate = 0.0;
                }

                // Insert Liquidation Data
                Some(liq_data) = self.additional_rx.rx_liquidation.recv() => {
                    // Update price
                    self.processed.event_type = Some("Liquidation".to_string());
                    self.processed.event_time = liq_data.event_time;
                    self.processed.processed_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;

                    self.processed.liquidation_side = liq_data.side;
                    self.processed.liquidation_dvolume = liq_data.quantity * liq_data.avg_price;

                    if self.tx_feature.send(self.processed.clone()).await.is_err() {
                        error!("Failed to send feature to executor");
                    }

                    // Re-Initialize liquidation data
                    self.processed.liquidation_side = "".to_string();
                    self.processed.liquidation_dvolume = 0.0;
                }
            }
        }
    }
}
