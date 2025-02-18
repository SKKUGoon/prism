use log::info;

use crate::prism::trade::{intra_exchange_param::IntraParams, LongShort};

pub struct SnipeLargeOrderStrategy {
    pub source: String,

    pub bid_price: Option<f32>,
    pub ask_price: Option<f32>,
    pub long_short: Option<LongShort>,

    large_order_threshold: f32,
}

impl SnipeLargeOrderStrategy {
    pub fn new(source: String, large_order_threshold: f32) -> Self {
        Self {
            source,
            bid_price: None,
            ask_price: None,
            long_short: None,
            large_order_threshold,
        }
    }

    pub fn evaluate(&mut self, data: &IntraParams) {
        let mut bid_quantity = f32::MIN;
        let mut ask_quantity = f32::MIN;
        let mut opportunity = false;

        data.bid_diff.iter().for_each(|(price, quantity)| {
            let price = price.parse::<f32>().unwrap_or(0.0);
            let quantity = quantity.parse::<f32>().unwrap_or(0.0);

            if price * quantity > self.large_order_threshold {
                bid_quantity = quantity;
                self.bid_price = Some(price); // TODO: Add slightly more than the price to ensure front running
                opportunity = true;
            }
        });

        data.ask_diff.iter().for_each(|(price, quantity)| {
            let price = price.parse::<f32>().unwrap_or(0.0);
            let quantity = quantity.parse::<f32>().unwrap_or(0.0);

            if price * quantity > self.large_order_threshold {
                ask_quantity = quantity;
                self.ask_price = Some(price); // TODO: Subtract slightly less than the price to ensure front running
                opportunity = true;
            }
        });

        // Follow the larger quantity
        if opportunity {
            if bid_quantity > ask_quantity {
                self.long_short = Some(LongShort::Long);
            } else {
                self.long_short = Some(LongShort::Short);
            }

            self.display(self.source.as_str());
        }
    }

    pub fn display(&self, source: &str) {
        info!(
            "Snipe Large Order Strategy - Exchange {}\n\
              Entry Price: {:?}\n\
              Long/Short: {:?}",
            source, self.bid_price, self.long_short
        );
    }
}
