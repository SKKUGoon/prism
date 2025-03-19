use rust_decimal::Decimal;

#[derive(Debug, Clone)]
pub struct Vwap {
    pub cumul_price_volume: Decimal,
    pub cumul_volume: Decimal,
}

impl Vwap {
    pub fn new() -> Self {
        Self {
            cumul_price_volume: Decimal::from(0),
            cumul_volume: Decimal::from(0),
        }
    }

    pub fn update(&mut self, price: Decimal, volume: Decimal) {
        self.cumul_price_volume += price * volume;
        self.cumul_volume += volume;
    }

    pub fn vwap(&self) -> Option<Decimal> {
        if self.cumul_volume == Decimal::from(0) {
            return None;
        }

        Some(self.cumul_price_volume / self.cumul_volume)
    }
}
