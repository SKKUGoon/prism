use super::Bar;
use rust_decimal::Decimal;

#[derive(Debug, Clone)]
pub struct TickImbalanceBar {
    pub bar: Bar,
}

impl TickImbalanceBar {
    pub fn new(ewma_tick_count_factor: f32, ewma_imbalance_factor: f32) -> Self {
        Self {
            bar: Bar::new(ewma_tick_count_factor, ewma_imbalance_factor),
        }
    }

    // pub fn genesis_bar(&mut self, mkt_data: &MarketData) -> Option<Self> {}
}
