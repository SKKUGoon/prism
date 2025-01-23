use crate::data::market::binance_aggtrade_future::MarketData;
use log::debug;
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct DollarImbalanceBar {
    // Volume Imbalance Bar
    pub id: String,
    pub ts: Option<u64>, // Time start, Timestamp
    pub te: Option<u64>, // Time end, Timestamp
    pub po: Option<f32>, // Price open
    pub ph: Option<f32>, // Price high
    pub pl: Option<f32>, // Price low
    pub pc: Option<f32>, // Price close
    pub imb: f32,        // Tick imbalance
    pub tsize: usize,    // Tick count
    pub volume_type: DollarVolumeType,

    // Constant
    genesis_collect_period: u64, // Cumulative time for creating the first bar
    ewma_factor: f32,

    // Threshold manager
    ewma_imb_current: f32,
    ewma_t_current: f32,
    historical_threshold: VecDeque<f32>,
    pub imb_thres: f32,
}

#[derive(Debug, Clone)]
pub enum DollarVolumeType {
    Maker,
    Taker,
    Both,
}

const DOLLAR_IMBALANCE_BAR_THRESHOLD_COUNT: usize = 50;

impl DollarImbalanceBar {
    pub fn new(volume_type: DollarVolumeType) -> Self {
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

            genesis_collect_period: 5000, // 5 seconds
            ewma_factor: 0.9, // Higher factor = more weights to recent data, more responsive to volatile market
            volume_type,
            ewma_imb_current: 0.0,
            ewma_t_current: 0.0,
            historical_threshold: VecDeque::new(),
            imb_thres: 0.0,
        }
    }

    pub fn genesis_bar(&mut self, mkt_data: &MarketData) -> Option<DollarImbalanceBar> {
        match self.ts {
            Some(ts) => {
                // Retrieve the most recent price from `pe`
                let prev_price = self.pc.unwrap(); // Guaranteed to be Some
                let price_change = mkt_data.price - prev_price;

                let tick_imbalance = match price_change.total_cmp(&0.0) {
                    std::cmp::Ordering::Greater => 1.0, // Price increase - buyer motivated
                    std::cmp::Ordering::Less => -1.0,   // Price decrease - seller motivated
                    std::cmp::Ordering::Equal => 0.0,   // Price stable - matched orders
                };

                let should_update = match self.volume_type {
                    DollarVolumeType::Maker => mkt_data.buyer_market_maker,
                    DollarVolumeType::Taker => !mkt_data.buyer_market_maker,
                    DollarVolumeType::Both => true,
                };

                if should_update {
                    self.imb += tick_imbalance * mkt_data.quantity * mkt_data.price;
                    self.tsize += 1;
                }

                // Update existing bar
                self.te = Some(mkt_data.time);
                self.pc = Some(mkt_data.price);
                self.ph = Some(self.ph.unwrap().max(mkt_data.price));
                self.pl = Some(self.pl.unwrap().min(mkt_data.price));

                if let Some(te) = self.te {
                    if (te - ts >= self.genesis_collect_period)
                        && (self.imb / self.tsize as f32 != 0.0)
                    {
                        // Create new bar
                        let b_t = self.imb / self.tsize as f32;
                        self.ewma_imb_current = b_t * self.ewma_factor
                            + (1.0 - self.ewma_factor) * self.ewma_imb_current; // EWMA_t = lambda * IMB_t + (1 - lambda) * EWMA_t-1
                        self.ewma_t_current = self.tsize as f32 * self.ewma_factor
                            + (1.0 - self.ewma_factor) * self.ewma_t_current; // EWMA_t = lambda * t_t + (1 - lambda) * EWMA_t-1

                        let threshold = self.ewma_imb_current.abs() * self.ewma_t_current;

                        self.historical_threshold.push_back(threshold);
                        self.imb_thres = threshold;

                        debug!("Genesis Volume Imbalance Bar Created");
                        return Some(self.clone());
                    }
                }
            }

            None => {
                // Create new bar
                self.ts = Some(mkt_data.time);
                self.te = Some(mkt_data.time);
                self.po = Some(mkt_data.price);
                self.ph = Some(mkt_data.price);
                self.pl = Some(mkt_data.price);
                self.pc = Some(mkt_data.price);
                self.tsize = 1;
                self.imb = 0.0;
            }
        }

        None
    }

    fn threshold_decay(&mut self, initial_value: f32) -> f32 {
        let (k1, k2) = (0.0001, 0.01);
        if self.tsize <= 5000usize {
            initial_value * (-k1 * (self.tsize as f32).sqrt()).exp() // Slow decay for t <= 5000
        } else {
            initial_value * (-k2 * ((self.tsize as f32) - 5000.0).sqrt()).exp() // Faster decay after 5000
        }
    }

    pub fn bar(&mut self, mkt_data: &MarketData) -> Option<DollarImbalanceBar> {
        match self.ts {
            Some(_) => {
                let prev_price = self.pc.unwrap(); // Guaranteed to be Some

                let price_change = mkt_data.price - prev_price;
                let tick_imbalance = match price_change.total_cmp(&0.0) {
                    std::cmp::Ordering::Greater => 1.0, // Price increase - buyer motivated
                    std::cmp::Ordering::Less => -1.0,   // Price decrease - seller motivated
                    std::cmp::Ordering::Equal => 0.0,   // Price stable - matched orders
                };

                let should_update = match self.volume_type {
                    DollarVolumeType::Maker => mkt_data.buyer_market_maker,
                    DollarVolumeType::Taker => !mkt_data.buyer_market_maker,
                    DollarVolumeType::Both => true,
                };

                if should_update {
                    self.imb += tick_imbalance * mkt_data.quantity * mkt_data.price;
                    self.tsize += 1;
                }

                // Update existing bar
                self.te = Some(mkt_data.time);
                self.pc = Some(mkt_data.price);
                self.ph = Some(self.ph.unwrap().max(mkt_data.price));
                self.pl = Some(self.pl.unwrap().min(mkt_data.price));

                let mut threshold = self.ewma_imb_current.abs() * self.ewma_t_current;

                // Manually set a threshold's limit to prevent the threshold explosion
                let threshold_max = self
                    .historical_threshold
                    .iter()
                    .max_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap_or(&threshold)
                    * 1.5;

                let threshold_min = self
                    .historical_threshold
                    .iter()
                    .min_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap_or(&threshold)
                    * 0.5;

                threshold = threshold.min(threshold_max).max(threshold_min);

                debug!(
                    "Volume Imbalance Bar: thres: {:?} decay: {:?} | imb: {:?}",
                    threshold,
                    self.threshold_decay(threshold),
                    self.imb
                );

                if self.imb.abs() >= self.threshold_decay(threshold) {
                    // Record new EWMA
                    let b_t = self.imb / self.tsize as f32;
                    self.ewma_imb_current =
                        b_t * self.ewma_factor + (1.0 - self.ewma_factor) * self.ewma_imb_current; // EWMA_t = lambda * IMB_t + (1 - lambda) * EWMA_t-1
                    self.ewma_t_current = self.tsize as f32 * self.ewma_factor
                        + (1.0 - self.ewma_factor) * self.ewma_t_current; // EWMA_t = lambda * t_t + (1 - lambda) * EWMA_t-1

                    // Update historical threshold
                    if self.historical_threshold.len() >= DOLLAR_IMBALANCE_BAR_THRESHOLD_COUNT {
                        self.historical_threshold.pop_front();
                    }
                    self.historical_threshold.push_back(threshold);

                    // Update imb_thres
                    self.imb_thres = self.threshold_decay(threshold);

                    // Create new bar
                    return Some(self.clone());
                }
            }
            None => {
                // Start of new bar
                self.ts = Some(mkt_data.time);
                self.te = Some(mkt_data.time);
                self.po = Some(mkt_data.price);
                self.ph = Some(mkt_data.price);
                self.pl = Some(mkt_data.price);
                self.pc = Some(mkt_data.price);
                self.tsize = 1;
                self.imb = 0.0;
            }
        }

        None
    }

    pub fn reset(&mut self) {
        // Retain the threshold and base threshold
        self.id = uuid::Uuid::new_v4().to_string();
        self.ts = None;
        self.te = None;
        self.po = None;
        self.ph = None;
        self.pl = None;
        self.pc = None;
        self.imb = 0.0;
        self.tsize = 0;
    }
}
