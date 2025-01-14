use log::info;

use crate::data::market::binance_aggtrade_future::MarketData;
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct Tib {
    // Tick Imbalance Bar
    pub id: String,
    pub ts: u64,                 // Time start, Timestamp
    pub te: u64,                 // Time end, Timestamp
    pub ps: Option<f32>,         // Price start
    pub pe: Option<f32>,         // Price end
    pub imb: f32,                // Tick imbalance
    thres: f32,        // Threshold for tick imbalance. Cross the threshold, generate a bar
    base_thres: f32,   // Base threshold for initial calibration
    vol_window: usize, // Lookback window for volatility estimation
    tick_changes: VecDeque<f32>, // Store recent tick changes for volatility estimation
}

#[allow(dead_code)]
impl Tib {
    pub fn new(base_thres: f32, vol_window: usize) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            ts: 0,
            te: 0,
            ps: None,
            pe: None,
            imb: 0.0,
            thres: base_thres,
            base_thres,
            vol_window,
            tick_changes: VecDeque::with_capacity(vol_window), // Initialize fixed size vector
        }
    }

    pub fn update(&mut self, mkt_data: &MarketData) -> Option<Tib> {
        if self.ts == 0 {
            self.ts = mkt_data.time;
            self.ps = Some(mkt_data.price);
            return None;
        }

        let prev_price = self.pe.unwrap_or(self.ps.unwrap());
        let price_change = mkt_data.price - prev_price;

        self.tick_changes.push_back(price_change);
        if self.tick_changes.len() > self.vol_window {
            self.tick_changes.pop_front();
        }

        // Dynamically adjust threshold based on volatility
        if self.tick_changes.len() >= self.vol_window {
            let mean = self.tick_changes.iter().sum::<f32>() / self.vol_window as f32;
            let vol = self
                .tick_changes
                .iter()
                .map(|x| (x - mean).powi(2))
                .sum::<f32>()
                / self.vol_window as f32;
            let std_dev = vol.sqrt();
            self.thres = self.base_thres * std_dev;
            info!("Threshold Updated to: {}", self.thres);
        }

        let tick_imbalance = if price_change > 0.0 {
            1.0
        } else if price_change < 0.0 {
            -1.0
        } else {
            0.0
        };

        self.imb += tick_imbalance;

        if self.imb.abs() >= self.thres {
            let complete_bar = Tib {
                id: self.id.clone(),
                ts: self.ts,
                te: mkt_data.time,
                ps: self.ps,
                pe: Some(mkt_data.price),
                imb: self.imb,
                thres: self.thres,
                base_thres: self.base_thres,
                vol_window: self.vol_window,
                tick_changes: VecDeque::new(), // No need to carry over tick changes
            };

            Some(complete_bar)
        } else {
            // Update current bar's ending value
            self.te = mkt_data.time;
            self.pe = Some(mkt_data.price);
            None
        }
    }

    pub fn reset(&mut self) {
        // Retain the threshold and base threshold
        self.id = uuid::Uuid::new_v4().to_string();
        self.ts = 0;
        self.te = 0;
        self.ps = None;
        self.pe = None;
        self.imb = 0.0;
    }
}
