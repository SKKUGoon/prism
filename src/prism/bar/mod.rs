use crate::data::market::MarketData;
use std::collections::VecDeque;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Bar {
    pub id: String,
    pub ts: Option<u64>, // Time start, Timestamp
    pub te: Option<u64>, // Time end, Timestamp
    pub po: Option<f32>, // Price open
    pub ph: Option<f32>, // Price high
    pub pl: Option<f32>, // Price low
    pub pc: Option<f32>, // Price close
    pub imb: f32,        // Imbalance
    pub tsize: usize,    // Tick count

    // Constant
    genesis_collect_period: u64,
    ewma_factor: f32,

    // Threshold manager
    ewma_imb_current: f32,
    ewma_t_current: f32,
    historical_threshold: VecDeque<f32>,
    pub imb_thres: f32,

    // VWAP
    pub vwap: f32,
    cum_price_volume: f32,
    cum_volume: f32,

    // Config
    candle_opened: bool,
}

#[allow(dead_code)]
impl Bar {
    pub fn new() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            ts: None,
            te: None,
            po: None,
            ph: None,
            pl: None,
            pc: None,
            imb: 0.0,
            tsize: 0,
            imb_thres: 0.0,
            vwap: 0.0,
            genesis_collect_period: 5000,
            ewma_factor: 0.9,
            ewma_imb_current: 0.0,
            ewma_t_current: 0.0,
            historical_threshold: VecDeque::new(),
            cum_price_volume: 0.0,
            cum_volume: 0.0,
            candle_opened: false,
        }
    }

    pub fn aggressive(&self) -> f32 {
        // Total amount of ticks / Duration to create the bar
        if let (Some(ts), Some(te)) = (self.ts, self.te) {
            if te - ts > 0 {
                self.tsize as f32 / (te - ts) as f32
            } else {
                0.0
            }
        } else {
            0.0
        }
    }

    pub fn aggressive_vol(&self) -> f32 {
        // Total amount of ticks * Cumul Product of price and volume / Duration to create the bar
        // Identifies false transactions (small but frequency transaction signals)
        if let (Some(ts), Some(te)) = (self.ts, self.te) {
            if te - ts > 0 {
                (self.tsize as f32 * self.cum_price_volume) / (te - ts) as f32
            } else {
                0.0
            }
        } else {
            0.0
        }
    }

    fn threshold_decay(&mut self, initial_value: f32) -> f32 {
        let (k1, k2) = (0.0001, 0.01);
        if self.tsize <= 5000usize {
            initial_value * (-k1 * (self.tsize as f32).sqrt()).exp()
        } else {
            initial_value * (-k2 * ((self.tsize as f32) - 5000.0).sqrt()).exp()
        }
    }

    fn update_vwap(&mut self, mkt_data: &MarketData) {
        self.cum_price_volume += mkt_data.price * mkt_data.quantity;
        self.cum_volume += mkt_data.quantity;

        if self.cum_volume > 0.0 {
            self.vwap = self.cum_price_volume / self.cum_volume;
        }
    }

    pub fn reset(&mut self) {
        self.id = uuid::Uuid::new_v4().to_string();
        self.ts = None;
        self.te = None;
        self.po = None;
        self.ph = None;
        self.pl = None;
        // Do not reset pc - previous price
        self.imb = 0.0;
        self.tsize = 0;
        self.cum_price_volume = 0.0;
        self.cum_volume = 0.0;
    }
}

// Trait for different bar implementations
pub trait BarImpl: Sized {
    fn calculate_imbalance(&self, mkt_data: &MarketData, prev_price: f32) -> f32;
    fn should_update(&self, mkt_data: &MarketData) -> bool;
    fn threshold_count(&self) -> usize;
}

pub trait VolumeDelta {
    fn calculate_cvd(&mut self, mkt_data: &MarketData, prev_price: f32);
}

// Re-export specific bar types
pub mod dollar_imbalance;
pub mod manager;
pub mod tick_imbalance;
pub mod volume_imbalance;

pub use dollar_imbalance::DollarImbalanceBar;
pub use tick_imbalance::TickImbalanceBar;
pub use volume_imbalance::VolumeImbalanceBar;
