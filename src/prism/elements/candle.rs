use rust_decimal::Decimal;

use super::vwap::Vwap;

#[derive(Debug, Clone)]
pub struct Candle {
    pub ts: Option<u64>,
    pub te: Option<u64>,

    pub po: Option<Decimal>,
    pub ph: Option<Decimal>,
    pub pl: Option<Decimal>,
    pub pc: Option<Decimal>,

    pub vwap: Vwap,

    pub buy_volume: Decimal,
    pub sell_volume: Decimal,

    pub tick_count: usize, // Now many ticks were included in the candle
}

impl Candle {
    pub fn new() -> Self {
        Self {
            ts: None,
            te: None,

            po: None,
            ph: None,
            pl: None,
            pc: None,
            vwap: Vwap::new(),

            buy_volume: Decimal::from(0),
            sell_volume: Decimal::from(0),

            tick_count: 0,
        }
    }

    pub fn update(&mut self, timestamp: u64, price: Decimal, volume: Decimal, sell_side: bool) {
        // Update Time Extremes
        if self.ts.is_none() {
            self.ts = Some(timestamp);
        }
        self.te = Some(timestamp);

        // Update Price Extremes
        if self.po.is_none() {
            self.po = Some(price);
        }
        if price >= self.ph.unwrap_or(price) {
            self.ph = Some(price);
        }
        if price <= self.pl.unwrap_or(price) {
            self.pl = Some(price);
        }
        self.pc = Some(price);

        // Update Volume Extremes
        if sell_side {
            self.sell_volume += volume;
        } else {
            self.buy_volume += volume;
        }

        // Update VWAP for this candle
        self.vwap.update(price, volume);

        // Update Tick Count
        self.tick_count += 1;
    }

    pub fn close(&mut self) {}
}
