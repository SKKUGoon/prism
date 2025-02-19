use crate::prism::{bar::manager::BarManager, stream::FeatureProcessed};
use std::collections::HashMap;

pub struct IntraParams {
    pub price: Option<f32>,
    pub data_time: Option<u64>,
    // 1. Time bar
    pub vwap: Option<f32>,
    // 2. Tick bar
    pub tick_vwap: Option<f32>,
    pub tick_imbalance: Option<f32>,
    pub tick_imbalance_thres: Option<f32>,
    // 3. Volume bar
    pub volume_vwap: Option<f32>,
    pub volume_cvd: Option<f32>,
    pub volume_imbalance: Option<f32>,
    pub volume_imbalance_thres: Option<f32>,
    // 4. Dollar bar
    pub dollar_vwap: Option<f32>,
    pub dollar_cvd: Option<f32>,
    pub dollar_imbalance: Option<f32>,
    pub dollar_imbalance_thres: Option<f32>,
    // 5. Bar History Manager
    pub bars: BarManager,
    // 6. Orderbook
    pub bid_diff: HashMap<String, String>,
    pub ask_diff: HashMap<String, String>,
}

impl IntraParams {
    pub fn new(historical_bars: usize) -> Self {
        Self {
            price: None,
            data_time: None,
            vwap: None,
            tick_vwap: None,
            tick_imbalance: None,
            tick_imbalance_thres: None,
            volume_vwap: None,
            volume_cvd: None,
            volume_imbalance: None,
            volume_imbalance_thres: None,
            dollar_vwap: None,
            dollar_cvd: None,
            dollar_imbalance: None,
            dollar_imbalance_thres: None,
            bars: BarManager::new(historical_bars),
            bid_diff: HashMap::new(),
            ask_diff: HashMap::new(),
        }
    }

    pub fn update_bars(&mut self, data: &FeatureProcessed) {
        self.bars
            .update_tick_imbalance_bar(&data.tick_imbalance_bar);
        self.bars
            .update_volume_imbalance_bar(&data.volume_imbalance_bar);
        self.bars
            .update_dollar_imbalance_bar(&data.dollar_imbalance_bar);
    }

    pub fn update_params(&mut self, data: &FeatureProcessed) {
        self.price = Some(data.price);
        self.data_time = Some(data.trade_time);
        self.vwap = None;

        self.tick_vwap = Some(data.tick_imbalance_vwap);
        self.tick_imbalance = Some(data.tick_imbalance);
        self.tick_imbalance_thres = Some(data.tick_imbalance_thres);

        self.volume_vwap = Some(data.volume_imbalance_vwap);
        self.volume_cvd = Some(data.volume_imbalance_bar.cvd);
        self.volume_imbalance = Some(data.volume_imbalance);
        self.volume_imbalance_thres = Some(data.volume_imbalance_thres);

        self.dollar_vwap = Some(data.dollar_imbalance_vwap);
        self.dollar_cvd = Some(data.dollar_imbalance_bar.cvd);
        self.dollar_imbalance = Some(data.dollar_imbalance);
        self.dollar_imbalance_thres = Some(data.dollar_imbalance_thres);

        self.bid_diff = data.near_price_bids.clone();
        self.ask_diff = data.near_price_asks.clone();
    }
}
