use crate::data::depth::OrderbookUpdateStream;
use rust_decimal::prelude::FromStr;
use rust_decimal::Decimal;
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct Orderbook {
    pub bids: BTreeMap<Decimal, Decimal>,
    pub asks: BTreeMap<Decimal, Decimal>,

    pub trade_time: u64,
    pub event_time: u64,
    pub last_source: Option<String>,
}

impl Orderbook {
    pub fn new() -> Self {
        Self {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            trade_time: 0,
            event_time: 0,
            last_source: None,
        }
    }

    pub async fn update(&mut self, update: &OrderbookUpdateStream) {
        if update.trade_time == 0 || update.event_time == 0 {
            return;
        }

        for bid in update.bids.clone() {
            self.update_bid(&bid.0, &bid.1);
        }

        for ask in update.asks.clone() {
            self.update_ask(&ask.0, &ask.1);
        }

        self.trade_time = update.trade_time;
        self.event_time = update.event_time;
        self.last_source = Some(update.last_update_exchange.clone());
    }

    fn update_bid(&mut self, price: &str, volume: &str) {
        let price = Decimal::from_str(price).unwrap();
        let volume = Decimal::from_str(volume).unwrap();

        if volume.is_zero() {
            self.bids.remove(&price);
        } else {
            self.bids.insert(price, volume);
        }
    }

    fn update_ask(&mut self, price: &str, volume: &str) {
        let price = Decimal::from_str(price).unwrap();
        let volume = Decimal::from_str(volume).unwrap();

        if volume.is_zero() {
            self.asks.remove(&price);
        } else {
            self.asks.insert(price, volume);
        }
    }

    pub fn best_bid(&self) -> Option<(Decimal, Decimal)> {
        self.bids
            .last_key_value()
            .map(|(price, volume)| (*price, *volume))
    }

    pub fn best_ask(&self) -> Option<(Decimal, Decimal)> {
        self.asks
            .first_key_value()
            .map(|(price, volume)| (*price, *volume))
    }
}
