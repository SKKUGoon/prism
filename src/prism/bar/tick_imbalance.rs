use crate::data::market::binance_aggtrade_future::MarketData;
use crate::prism::bar::{Bar, BarImpl};

#[derive(Debug, Clone)]
pub struct TickImbalanceBar {
    pub bar: Bar,
}

impl TickImbalanceBar {
    pub fn new() -> Self {
        Self { bar: Bar::new() }
    }

    pub fn genesis_bar(&mut self, mkt_data: &MarketData) -> Option<Self> {
        match self.bar.ts {
            Some(ts) => {
                // Retrieve the most recent price from `pe`
                let prev_price = self.bar.pc.unwrap(); // Guaranteed to be Some
                let tick_imbalance = self.calculate_imbalance(mkt_data, prev_price);
                self.bar.imb += tick_imbalance; // Cumulation of tick imbalances
                self.bar.tsize += 1;

                // Update existing bar
                self.bar.te = Some(mkt_data.trade_time);
                self.bar.pc = Some(mkt_data.price);
                self.bar.ph = Some(self.bar.ph.unwrap().max(mkt_data.price));
                self.bar.pl = Some(self.bar.pl.unwrap().min(mkt_data.price));

                // Update VWAP
                self.bar.update_vwap(mkt_data);

                // Check if the bar is ready to be created
                if let Some(te) = self.bar.te {
                    // Genesis bar creation is done after pre-adjusted amount of time
                    if (te - ts >= self.bar.genesis_collect_period)
                        && (self.bar.imb / self.bar.tsize as f32 != 0.0)
                    {
                        // Create new bar
                        let b_t = self.bar.imb / self.bar.tsize as f32;
                        self.bar.ewma_imb_current = b_t * self.bar.ewma_factor
                            + (1.0 - self.bar.ewma_factor) * self.bar.ewma_imb_current; // EWMA_t = lambda * IMB_t + (1 - lambda) * EWMA_t-1
                        self.bar.ewma_t_current = self.bar.tsize as f32 * self.bar.ewma_factor
                            + (1.0 - self.bar.ewma_factor) * self.bar.ewma_t_current; // EWMA_t = lambda * t_t + (1 - lambda) * EWMA_t-1

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
                let prev_price = self.bar.pc.unwrap(); // Guaranteed to be Some
                let tick_imbalance = self.calculate_imbalance(mkt_data, prev_price);
                self.bar.imb += tick_imbalance;
                self.bar.tsize += 1;

                // Update existing bar
                self.bar.te = Some(mkt_data.trade_time);
                self.bar.pc = Some(mkt_data.price);
                self.bar.ph = Some(self.bar.ph.unwrap().max(mkt_data.price));
                self.bar.pl = Some(self.bar.pl.unwrap().min(mkt_data.price));

                // Update VWAP
                self.bar.update_vwap(mkt_data);

                // Calculate threshold
                // Manually set a threshold's limit to prevent the threshold explosion
                let mut threshold = self.bar.ewma_imb_current.abs() * self.bar.ewma_t_current;

                let threshold_max = self
                    .bar
                    .historical_threshold
                    .clone()
                    .iter()
                    .max_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap_or(&threshold)
                    * 1.5;

                let threshold_min = self
                    .bar
                    .historical_threshold
                    .clone()
                    .iter()
                    .min_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap_or(&threshold)
                    * 0.5;

                threshold = threshold.min(threshold_max).max(threshold_min);

                if self.bar.imb.abs() >= self.bar.threshold_decay(threshold) {
                    // Record new EWMA
                    let b_t = self.bar.imb / self.bar.tsize as f32;
                    self.bar.ewma_imb_current = b_t * self.bar.ewma_factor
                        + (1.0 - self.bar.ewma_factor) * self.bar.ewma_imb_current; // EWMA_t = lambda * IMB_t + (1 - lambda) * EWMA_t-1
                    self.bar.ewma_t_current = self.bar.tsize as f32 * self.bar.ewma_factor
                        + (1.0 - self.bar.ewma_factor) * self.bar.ewma_t_current; // EWMA_t = lambda * t_t + (1 - lambda) * EWMA_t-1

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
                self.bar.imb = 0.0;
                self.bar.tsize = 1;

                // Reset VWAP
                self.bar.cum_price_volume = 0.0;
                self.bar.cum_volume = 0.0;
                self.bar.vwap = 0.0;
            }
        }

        None
    }

    pub fn reset(&mut self) {
        self.bar.reset();
    }
}

impl BarImpl for TickImbalanceBar {
    fn calculate_imbalance(&self, mkt_data: &MarketData, prev_price: f32) -> f32 {
        let price_change = mkt_data.price - prev_price;
        match price_change.total_cmp(&0.0) {
            std::cmp::Ordering::Greater => 1.0,
            std::cmp::Ordering::Less => -1.0,
            std::cmp::Ordering::Equal => 0.0,
        }
    }

    fn should_update(&self, _mkt_data: &MarketData) -> bool {
        true // Tick imbalance always updates
    }

    fn threshold_count(&self) -> usize {
        50
    }
}
