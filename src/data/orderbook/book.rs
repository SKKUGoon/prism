use std::collections::HashMap;
use tokio::sync::mpsc::{Receiver, Sender};

use log::debug;

#[derive(Debug)]
#[allow(dead_code)]
pub struct Orderbook {
    pub bids: HashMap<String, String>, // key: price, value: order id
    pub asks: HashMap<String, String>, // key: price, value: order id

    pub time: u64,
    pub last_update_exchange: String,

    rx: Receiver<OrderbookUpdateStream>,
    tx: Sender<OrderbookData>,
}

#[derive(Debug, Clone)]
pub struct OrderbookData {
    pub bids: HashMap<String, String>, // key: price, value: order id
    pub asks: HashMap<String, String>, // key: price, value: order id
    pub time: u64,
}

pub struct OrderbookUpdateStream {
    pub bids: HashMap<String, String>, // key: price, value: order id
    pub asks: HashMap<String, String>, // key: price, value: order id

    pub time: u64,
    pub last_update_exchange: String,
}

#[allow(dead_code)]
impl Orderbook {
    pub fn new(rx: Receiver<OrderbookUpdateStream>, tx: Sender<OrderbookData>) -> Self {
        Self {
            bids: HashMap::new(),
            asks: HashMap::new(),
            time: 0,
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
                bids: self.bids.clone(),
                asks: self.asks.clone(),
                time: self.time,
            };

            if let Err(e) = self.tx.send(data).await {
                log::error!("Error sending orderbook data to prism: {}", e);
            }

            self.display();
        }
    }

    pub fn display(&self) {
        debug!(
            "Time: {} | Source: {}",
            self.time, self.last_update_exchange
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

        self.time = update.time;
        self.last_update_exchange = update.last_update_exchange.to_string();
    }
}

impl OrderbookData {
    pub fn orderbook_imbalance(&mut self, price: f32, margin: f32) -> f32 {
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
