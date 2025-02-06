use crate::data::market::binance_aggtrade_future::MarketData;
use crate::prism::bar::{Bar, BarImpl, VolumeDelta};

#[derive(Debug, Clone)]
pub struct VolumeImbalanceBar {
    pub bar: Bar,
    pub cvd: f32,
}

impl VolumeImbalanceBar {
    pub fn new() -> Self {
        Self {
            bar: Bar::new(),
            cvd: 0.0,
        }
    }

    pub fn genesis_bar(&mut self, mkt_data: &MarketData) -> Option<Self> {
        match self.bar.ts {
            Some(ts) => {
                let prev_price = self.bar.pc.unwrap();

                if self.should_update(mkt_data) {
                    let imbalance = self.calculate_imbalance(mkt_data, prev_price);
                    self.bar.imb += imbalance;
                    self.bar.tsize += 1;
                }

                // Update VWAP
                self.bar.update_vwap(mkt_data);

                // Update CVD
                self.calculate_cvd(mkt_data, prev_price);

                // Update bar data
                self.bar.te = Some(mkt_data.trade_time);
                self.bar.pc = Some(mkt_data.price);
                self.bar.ph = Some(self.bar.ph.unwrap().max(mkt_data.price));
                self.bar.pl = Some(self.bar.pl.unwrap().min(mkt_data.price));

                if let Some(te) = self.bar.te {
                    if (te - ts >= self.bar.genesis_collect_period)
                        && (self.bar.imb / self.bar.tsize as f32 != 0.0)
                    {
                        let b_t = self.bar.imb / self.bar.tsize as f32;
                        self.bar.ewma_imb_current = b_t * self.bar.ewma_factor
                            + (1.0 - self.bar.ewma_factor) * self.bar.ewma_imb_current;
                        self.bar.ewma_t_current = self.bar.tsize as f32 * self.bar.ewma_factor
                            + (1.0 - self.bar.ewma_factor) * self.bar.ewma_t_current;

                        let threshold = self.bar.ewma_imb_current.abs() * self.bar.ewma_t_current;
                        self.bar.historical_threshold.push_back(threshold);
                        self.bar.imb_thres = self.bar.threshold_decay(threshold);

                        return Some(self.clone());
                    }
                }
            }
            None => {
                // Initialize new bar
                self.bar.ts = Some(mkt_data.trade_time);
                self.bar.te = Some(mkt_data.trade_time);
                self.bar.po = Some(mkt_data.price);
                self.bar.ph = Some(mkt_data.price);
                self.bar.pl = Some(mkt_data.price);
                self.bar.pc = Some(mkt_data.price);
                self.bar.tsize = 1;
                self.bar.imb = 0.0;
                self.bar.vwap = 0.0;
                self.bar.cum_price_volume = 0.0;
                self.bar.cum_volume = 0.0;
            }
        }
        None
    }

    pub fn bar(&mut self, mkt_data: &MarketData) -> Option<Self> {
        match self.bar.ts {
            Some(_) => {
                let prev_price = self.bar.pc.unwrap();

                if self.should_update(mkt_data) {
                    let imbalance = self.calculate_imbalance(mkt_data, prev_price);
                    self.bar.imb += imbalance;
                    self.bar.tsize += 1;
                }

                // Update VWAP
                self.bar.update_vwap(mkt_data);

                // Update CVD
                self.calculate_cvd(mkt_data, prev_price);

                // Update bar data
                self.bar.te = Some(mkt_data.trade_time);
                self.bar.pc = Some(mkt_data.price);
                self.bar.ph = Some(self.bar.ph.unwrap().max(mkt_data.price));
                self.bar.pl = Some(self.bar.pl.unwrap().min(mkt_data.price));

                let mut threshold = self.bar.ewma_imb_current.abs() * self.bar.ewma_t_current;

                let threshold_max = self
                    .bar
                    .historical_threshold
                    .iter()
                    .max_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap_or(&threshold)
                    * 1.5;

                let threshold_min = self
                    .bar
                    .historical_threshold
                    .iter()
                    .min_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap_or(&threshold)
                    * 0.5;

                threshold = threshold.min(threshold_max).max(threshold_min);

                if self.bar.imb.abs() >= self.bar.threshold_decay(threshold) {
                    // Record new EWMA
                    let b_t = self.bar.imb / self.bar.tsize as f32;
                    self.bar.ewma_imb_current = b_t * self.bar.ewma_factor
                        + (1.0 - self.bar.ewma_factor) * self.bar.ewma_imb_current;
                    self.bar.ewma_t_current = self.bar.tsize as f32 * self.bar.ewma_factor
                        + (1.0 - self.bar.ewma_factor) * self.bar.ewma_t_current;

                    // Update historical threshold
                    if self.bar.historical_threshold.len() >= self.threshold_count() {
                        self.bar.historical_threshold.pop_front();
                    }
                    self.bar.historical_threshold.push_back(threshold);
                    self.bar.imb_thres = self.bar.threshold_decay(threshold);

                    // Create new bar
                    return Some(self.clone());
                }
            }
            None => {
                self.bar.ts = Some(mkt_data.trade_time);
                self.bar.te = Some(mkt_data.trade_time);
                self.bar.po = Some(mkt_data.price);
                self.bar.ph = Some(mkt_data.price);
                self.bar.pl = Some(mkt_data.price);
                self.bar.pc = Some(mkt_data.price);
                self.bar.tsize = 1;
                self.bar.imb = 0.0;

                // Reset VWAP
                self.bar.vwap = 0.0;
                self.bar.cum_price_volume = 0.0;
                self.bar.cum_volume = 0.0;
            }
        }
        None
    }

    pub fn reset(&mut self) {
        self.bar.reset();
    }
}

impl BarImpl for VolumeImbalanceBar {
    fn calculate_imbalance(&self, mkt_data: &MarketData, prev_price: f32) -> f32 {
        let price_change = mkt_data.price - prev_price;
        let tick_imbalance = match price_change.total_cmp(&0.0) {
            std::cmp::Ordering::Greater => 1.0,
            std::cmp::Ordering::Less => -1.0,
            std::cmp::Ordering::Equal => 0.0,
        };

        tick_imbalance * mkt_data.quantity
    }

    fn should_update(&self, _mkt_data: &MarketData) -> bool {
        true
    }

    fn threshold_count(&self) -> usize {
        50
    }
}

impl VolumeDelta for VolumeImbalanceBar {
    fn calculate_cvd(&mut self, mkt_data: &MarketData, prev_price: f32) {
        let price_change = mkt_data.price - prev_price;
        let tick_imbalance = match price_change.total_cmp(&0.0) {
            std::cmp::Ordering::Greater => 1.0,
            std::cmp::Ordering::Less => -1.0,
            std::cmp::Ordering::Equal => 0.0,
        };

        self.cvd += tick_imbalance * mkt_data.quantity * mkt_data.price
    }
}
