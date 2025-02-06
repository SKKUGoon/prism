use std::collections::HashMap;
use tokio::sync::mpsc::{Receiver, Sender};

use log::debug;

#[derive(Debug)]
#[allow(dead_code)]
pub struct Orderbook {
    pub bids: HashMap<String, String>, // key: price, value: order id
    pub asks: HashMap<String, String>, // key: price, value: order id

    pub trade_time: u64,
    pub event_time: u64,
    pub last_update_exchange: String,

    rx: Receiver<OrderbookUpdateStream>,
    tx: Sender<OrderbookData>,
}

#[derive(Debug, Clone)]
pub struct OrderbookData {
    best_bid: (String, String),
    best_ask: (String, String),

    pub bids: HashMap<String, String>, // key: price, value: order id
    pub asks: HashMap<String, String>, // key: price, value: order id
    pub trade_time: u64,
    pub event_time: u64,
}

pub struct OrderbookUpdateStream {
    pub bids: HashMap<String, String>, // key: price, value: order id
    pub asks: HashMap<String, String>, // key: price, value: order id

    pub trade_time: u64,
    pub event_time: u64,
    pub last_update_exchange: String,
}

#[allow(dead_code)]
impl Orderbook {
    pub fn new(rx: Receiver<OrderbookUpdateStream>, tx: Sender<OrderbookData>) -> Self {
        Self {
            bids: HashMap::new(),
            asks: HashMap::new(),
            trade_time: 0,
            event_time: 0,
            last_update_exchange: String::new(),
            rx,
            tx,
        }
    }

    pub async fn listen(&mut self) {
        while let Some(update) = self.rx.recv().await {
            self.update(update);

            // Send updated orderbook to prism
            let data = OrderbookData {
                best_bid: (String::new(), String::new()),
                best_ask: (String::new(), String::new()),
                bids: self.bids.clone(),
                asks: self.asks.clone(),
                trade_time: self.trade_time,
                event_time: self.event_time,
            };

            if data.trade_time == 0 || data.event_time == 0 {
                continue;
            }

            if let Err(e) = self.tx.send(data).await {
                log::error!("Error sending orderbook data to prism: {}", e);
            }

            self.display();
        }
    }

    pub fn display(&self) {
        debug!(
            "Time: {} | Source: {}",
            self.trade_time, self.last_update_exchange
        );
    }

    pub fn update(&mut self, update: OrderbookUpdateStream) {
        for (price, quantity) in update.bids.iter() {
            if quantity == "0" {
                self.bids.remove(price);
            } else {
                self.bids.insert(price.clone(), quantity.clone());
            }
        }

        for (price, quantity) in update.asks.iter() {
            if quantity == "0" {
                self.asks.remove(price);
            } else {
                self.asks.insert(price.clone(), quantity.clone());
            }
        }

        self.trade_time = update.trade_time;
        self.event_time = update.event_time;
        self.last_update_exchange = update.last_update_exchange.to_string();
    }
}

#[allow(dead_code)]
impl OrderbookData {
    pub fn orderflow_imbalance(&mut self) -> f32 {
        let best_bid = self
            .bids
            .iter()
            .filter(|(_, bq)| bq.parse::<f32>().unwrap_or(0f32) > 0f32)
            .max_by(|a, b| {
                a.0.parse::<f32>()
                    .unwrap_or(0f32)
                    .partial_cmp(&b.0.parse::<f32>().unwrap_or(0f32))
                    .unwrap()
            });

        let best_ask = self
            .asks
            .iter()
            .filter(|(_, aq)| aq.parse::<f32>().unwrap_or(0f32) > 0f32)
            .min_by(|a, b| {
                a.0.parse::<f32>()
                    .unwrap_or(f32::MAX)
                    .partial_cmp(&b.0.parse::<f32>().unwrap_or(f32::MAX))
                    .unwrap()
            });

        let prev_best_bid_p = self.best_bid.0.parse::<f32>().unwrap_or(0f32);
        let prev_best_bid_q = self.best_bid.1.parse::<f32>().unwrap_or(0f32);

        let prev_best_ask_p = self.best_ask.0.parse::<f32>().unwrap_or(0f32);
        let prev_best_ask_q = self.best_ask.1.parse::<f32>().unwrap_or(0f32);

        let new_best_bid_p = best_bid.unwrap().0.parse::<f32>().unwrap_or(0f32);
        let new_best_bid_q = best_bid.unwrap().1.parse::<f32>().unwrap_or(0f32);

        let new_best_ask_p = best_ask.unwrap().0.parse::<f32>().unwrap_or(0f32);
        let new_best_ask_q = best_ask.unwrap().1.parse::<f32>().unwrap_or(0f32);

        // Bid orderflow
        let bid_overflow = match new_best_bid_p - prev_best_bid_p {
            // New bid price is higher than previous bid price. All the volume is new volume flow
            x if x > 0f32 => new_best_bid_q,

            // New bid price is the same with previous bid price. Increased or Decreased volume is the new volume flow
            0f32 => new_best_bid_q - prev_best_bid_q,

            // New bid price is lower than previous bid price. All the volume is previous volume flow
            x if x < 0f32 => -prev_best_bid_q,
            _ => 0f32,
        };

        // Ask orderflow
        let ask_overflow = match new_best_ask_p - prev_best_ask_p {
            // New ask price is higher than previous ask price. All the volume is previous volume flow
            x if x > 0f32 => -prev_best_ask_q,

            // New ask price is the same with previous ask price. Increased or Decreased volume is the new volume flow
            0f32 => new_best_ask_q - prev_best_ask_q,

            // New ask price is lower than previous ask price. All the volume is new volume flow
            x if x < 0f32 => new_best_ask_q,
            _ => 0f32,
        };

        // Positive when there are more buying orders
        // Negative when there are more selling orders
        // Measures volume as well as directions
        bid_overflow - ask_overflow
    }

    pub fn orderbook_imbalance(&mut self) -> f32 {
        let best_bid = self
            .bids
            .iter()
            .filter(|(_, bq)| bq.parse::<f32>().unwrap_or(0f32) > 0f32)
            .max_by(|a, b| {
                a.0.parse::<f32>()
                    .unwrap_or(0f32)
                    .partial_cmp(&b.0.parse::<f32>().unwrap_or(0f32))
                    .unwrap()
            });

        let best_ask = self
            .asks
            .iter()
            .filter(|(_, aq)| aq.parse::<f32>().unwrap_or(0f32) > 0f32)
            .min_by(|a, b| {
                a.0.parse::<f32>()
                    .unwrap_or(f32::MAX)
                    .partial_cmp(&b.0.parse::<f32>().unwrap_or(f32::MAX))
                    .unwrap()
            });

        let best_bid_quantity: f32 = best_bid
            .map(|(_, bq)| bq.parse::<f32>().unwrap_or(0f32))
            .unwrap_or(0f32);
        let best_ask_quantity: f32 = best_ask
            .map(|(_, aq)| aq.parse::<f32>().unwrap_or(0f32))
            .unwrap_or(0f32);

        if best_bid_quantity + best_ask_quantity == 0f32 {
            0f32
        } else {
            (best_bid_quantity - best_ask_quantity) / (best_bid_quantity + best_ask_quantity)
        }
    }

    pub fn update_best_bid_ask(&mut self) {
        let best_bid = self
            .bids
            .iter()
            .filter(|(_, bq)| bq.parse::<f32>().unwrap_or(0f32) > 0f32)
            .max_by(|a, b| {
                a.0.parse::<f32>()
                    .unwrap_or(0f32)
                    .partial_cmp(&b.0.parse::<f32>().unwrap_or(0f32))
                    .unwrap()
            });

        let best_ask = self
            .asks
            .iter()
            .filter(|(_, aq)| aq.parse::<f32>().unwrap_or(0f32) > 0f32)
            .min_by(|a, b| {
                a.0.parse::<f32>()
                    .unwrap_or(f32::MAX)
                    .partial_cmp(&b.0.parse::<f32>().unwrap_or(f32::MAX))
                    .unwrap()
            });

        self.best_bid = (best_bid.unwrap().0.clone(), best_bid.unwrap().1.clone());
        self.best_ask = (best_ask.unwrap().0.clone(), best_ask.unwrap().1.clone());
    }

    pub fn orderbook_imbalance_slack(&mut self, price: f32, margin: f32) -> f32 {
        // Pre-calculate bounds
        let upper_bound = price * (1f32 + margin);
        let lower_bound = price * (1f32 - margin);

        // Iterate over bids
        let obi_bids: f32 = self
            .bids
            .iter()
            .filter(|(bp, _)| {
                let bp = bp.parse::<f32>().unwrap_or(0f32);
                bp >= lower_bound && bp <= upper_bound
            })
            .map(|(_, bq)| bq.parse::<f32>().unwrap_or(0f32))
            .sum();

        // Iterate over asks
        let obi_asks: f32 = self
            .asks
            .iter()
            .filter(|(ap, _)| {
                let ap = ap.parse::<f32>().unwrap_or(0f32);
                ap >= lower_bound && ap <= upper_bound
            })
            .map(|(_, aq)| aq.parse::<f32>().unwrap_or(0f32))
            .sum();

        // Avoid division by zero
        if obi_bids + obi_asks == 0f32 {
            0f32
        } else {
            (obi_bids - obi_asks) / (obi_bids + obi_asks)
        }
    }
}
