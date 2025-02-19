use crate::prism::trade::{intra_exchange_param::IntraParams, LongShort};

#[derive(Debug, Clone)]
pub struct SnipeLargeOrderParams {
    time: u64,
    price: f32,
    best_bid_price: f32,
    best_bid_volume: f32,
    best_ask_price: f32,
    best_ask_volume: f32,
    best_bid_ask_spread: f32,
    total_bid_volume: f32, // Sum of near-best bid volumes
    total_ask_volume: f32, // Sum of near-best ask volumes
    market_buy_size: f32,
    market_sell_size: f32,
}

pub struct SnipeLargeOrderStrategy {
    pub source: String,
    // TODO: Determine Large Number Threshold
    // pub bid_price: Option<f32>,
    // pub ask_price: Option<f32>,
    // pub long_short: Option<LongShort>,
}

impl SnipeLargeOrderStrategy {
    pub fn new(source: String) -> Self {
        Self {
            source,
            // bid_price: None,
            // ask_price: None,
            // long_short: None,
        }
    }

    pub fn evaluate(&mut self, data: &IntraParams) -> Option<SnipeLargeOrderParams> {
        // TODO: Remove
        if self.source != "Upbit Spot" {
            return None;
        }

        if data.bid_diff.is_empty() || data.ask_diff.is_empty() {
            return None;
        }

        // Convert bid_diff entries to vec for sorting
        let mut bid_entries: Vec<_> = data.bid_diff.iter().collect();
        let mut ask_entries: Vec<_> = data.ask_diff.iter().collect();

        // Sort by price in descending order
        bid_entries.sort_by(|(price1, _), (price2, _)| {
            let p1 = price1.parse::<f32>().unwrap_or(0.0);
            let p2 = price2.parse::<f32>().unwrap_or(0.0);
            p2.partial_cmp(&p1).unwrap()
        });

        // Sort by price in ascending order
        ask_entries.sort_by(|(price1, _), (price2, _)| {
            let p1 = price1.parse::<f32>().unwrap_or(0.0);
            let p2 = price2.parse::<f32>().unwrap_or(0.0);
            p1.partial_cmp(&p2).unwrap()
        });

        // Identify best bid price and volume
        let best_bid_price = bid_entries[0].0.parse::<f32>().unwrap_or(0.0);
        let best_bid_volume = bid_entries[0].1.parse::<f32>().unwrap_or(0.0);

        // Identify best ask price and volume
        let best_ask_price = ask_entries[0].0.parse::<f32>().unwrap_or(0.0);
        let best_ask_volume = ask_entries[0].1.parse::<f32>().unwrap_or(0.0);

        // Identify total bid and ask volume
        let total_bid_volume = bid_entries
            .iter()
            .map(|(_, volume)| volume.parse::<f32>().unwrap_or(0.0))
            .sum::<f32>();
        let total_ask_volume = ask_entries
            .iter()
            .map(|(_, volume)| volume.parse::<f32>().unwrap_or(0.0))
            .sum::<f32>();

        // Build `SnipeLargeOrderParams`
        let params = SnipeLargeOrderParams {
            time: data.data_time.unwrap_or(0),
            price: data.price.unwrap_or(0.0),
            best_bid_price,
            best_bid_volume,
            best_ask_price,
            best_ask_volume,
            best_bid_ask_spread: best_bid_price - best_ask_price,
            total_bid_volume,
            total_ask_volume,
            market_buy_size: -1.0,  // Indicates no data
            market_sell_size: -1.0, // Indicates no data
        };

        Some(params)
    }
}
