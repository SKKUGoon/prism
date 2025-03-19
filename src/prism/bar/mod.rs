use super::elements::candle::Candle;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;

pub const GENESIS_COLLECT_PERIOD: usize = 1000 * 1800; // 1800 seconds

#[derive(Debug, Clone)]
pub struct Bar {
    pub id: uuid::Uuid,
    pub candle: Candle,

    imbalance_path: Vec<Decimal>,

    pub threshold: f32,
    ewma_tick_count_factor: f32,
    tick_count: usize,
    ewma_imbalance_factor: f32,
    cumul_imbalance: f32,
}

impl Bar {
    pub fn new(ewma_tick_count_factor: f32, ewma_imbalance_factor: f32) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            candle: Candle::new(),

            imbalance_path: Vec::new(),

            threshold: 0.0,
            ewma_tick_count_factor,
            tick_count: 0,
            ewma_imbalance_factor,
            cumul_imbalance: 0.0,
        }
    }

    pub fn aggressive(&self) -> Option<f32> {
        if let (Some(ts), Some(te)) = (self.candle.ts, self.candle.te) {
            if te - ts > 0 {
                Some(self.candle.tick_count as f32 / (te - ts) as f32)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn aggressive_vol(&self) -> Option<f32> {
        if let (Some(ts), Some(te)) = (self.candle.ts, self.candle.te) {
            if te - ts > 0 {
                Some(
                    (self.candle.tick_count as f32)
                        * (self.candle.vwap.cumul_price_volume.to_f32().unwrap_or(0.0))
                        / (te - ts) as f32,
                )
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn reset(&mut self) {
        // Reset Bar except for `threshold`
        self.id = uuid::Uuid::new_v4();
        self.candle = Candle::new();
        self.imbalance_path.clear();
        self.cumul_imbalance = 0.0;
        self.tick_count = 0;
    }
}

pub trait BarBehavior {
    fn new(ewma_tick_count_factor: f32, ewma_imbalance_factor: f32) -> Self;
    fn aggressive(&self) -> Option<f32>;
    fn aggressive_vol(&self) -> Option<f32>;
    fn reset(&mut self);
}

pub trait BarImpl {
    fn initiate_bar(&mut self);
    fn update_bar(&mut self);
}

pub mod tib;

//     fn threshold_decay(&mut self, initial_value: f32) -> f32 {
//         let (k1, k2) = (0.0001, 0.01);
//         if self.tsize <= 5000usize {
//             initial_value * (-k1 * (self.tsize as f32).sqrt()).exp()
//         } else {
//             initial_value * (-k2 * ((self.tsize as f32) - 5000.0).sqrt()).exp()
//         }
//     }

// // Trait for different bar implementations
// pub trait BarImpl: Sized {
//     fn calculate_imbalance(&self, mkt_data: &MarketData, prev_price: f32) -> f32;
//     fn should_update(&self, mkt_data: &MarketData) -> bool;
//     fn threshold_count(&self) -> usize;
// }

// pub trait VolumeDelta {
//     fn calculate_cvd(&mut self, mkt_data: &MarketData, prev_price: f32);
// }

// // Re-export specific bar types
// pub mod dollar_imbalance;
// pub mod manager;
// pub mod tick_imbalance;
// pub mod volume_imbalance;

// pub use dollar_imbalance::DollarImbalanceBar;
// pub use tick_imbalance::TickImbalanceBar;
// pub use volume_imbalance::VolumeImbalanceBar;
