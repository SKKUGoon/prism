use crate::data::market::binance_aggtrade_future::MarketData;
use crate::prism::bar::{Bar, BarImpl, VolumeDelta};

#[derive(Debug, Clone)]
pub struct DollarImbalanceBar {
    pub bar: Bar,
    pub cvd: f32,
}

impl DollarImbalanceBar {
    pub fn new() -> Self {
        Self {
            bar: Bar::new(),
            cvd: 0.0,
        }
    }

    pub fn genesis_bar(&mut self, mkt_data: &MarketData) -> Option<Self> {
        let prev_price = self.bar.pc.unwrap_or(mkt_data.price);

        if self.should_update(mkt_data) {
            // Compare the current price with the previous price. Generate the tick imbalance
            let imbalance = self.calculate_imbalance(mkt_data, prev_price);
            self.bar.imb += imbalance * mkt_data.quantity * mkt_data.price;

            // Update the tick count
            self.bar.tsize += 1;
        }

        // Update bar parameters
        if !self.bar.candle_opened {
            self.bar.candle_opened = true;
            self.bar.ts = Some(mkt_data.trade_time);
        }

        self.bar.te = Some(mkt_data.trade_time);
        self.bar.pc = Some(mkt_data.price);
        self.bar.ph = Some(self.bar.ph.unwrap_or(mkt_data.price).max(mkt_data.price));
        self.bar.pl = Some(self.bar.pl.unwrap_or(mkt_data.price).min(mkt_data.price));

        // Update VWAP
        self.bar.update_vwap(mkt_data);

        // Update CVD - Only for Dollar Imbalance and Volume Imbalance
        self.calculate_cvd(mkt_data, prev_price);

        // Genesis bar creation is done after pre-adjusted amount of time
        if let (Some(ts), Some(te)) = (self.bar.ts, self.bar.te) {
            if (te - ts >= self.bar.genesis_collect_period)
                && (self.bar.imb / self.bar.tsize as f32 != 0.0)
            {
                let completed_bar = self.clone(); // Bar is completed - Return this bar

                // Update the parameters and prepare for new bar.
                // These parameters will be used for the NEXT BAR.
                let b_t = self.bar.imb / self.bar.tsize as f32;
                self.bar.ewma_imb_current = b_t * self.bar.ewma_factor
                    + (1.0 - self.bar.ewma_factor) * self.bar.ewma_imb_current; // EWMA_t = lambda * IMB_t + (1 - lambda) * EWMA_t-1
                self.bar.ewma_t_current = self.bar.tsize as f32 * self.bar.ewma_factor
                    + (1.0 - self.bar.ewma_factor) * self.bar.ewma_t_current; // EWMA_t = lambda * t_t + (1 - lambda) * EWMA_t-1
                let threshold = self.bar.ewma_imb_current.abs() * self.bar.ewma_t_current;

                self.bar.imb_thres = threshold;
                self.bar.candle_opened = false; // Close the candle

                return Some(completed_bar);
            }
        }

        None
    }

    pub fn bar(&mut self, mkt_data: &MarketData) -> Option<Self> {
        let prev_price = self.bar.pc.unwrap(); // Guaranteed to be Some

        if self.should_update(mkt_data) {
            let tick_imbalance = self.calculate_imbalance(mkt_data, prev_price);
            self.bar.imb += tick_imbalance * mkt_data.quantity * mkt_data.price;
            self.bar.tsize += 1;
        }

        // Update existing bar
        if !self.bar.candle_opened {
            self.bar.candle_opened = true;
            self.bar.ts = Some(mkt_data.trade_time);
        }

        self.bar.te = Some(mkt_data.trade_time);
        self.bar.pc = Some(mkt_data.price);
        self.bar.ph = Some(self.bar.ph.unwrap_or(mkt_data.price).max(mkt_data.price));
        self.bar.pl = Some(self.bar.pl.unwrap_or(mkt_data.price).min(mkt_data.price));

        // Manually decreased threshold to prevent the threshold explosion
        self.bar.imb_thres = self.bar.threshold_decay(self.bar.imb_thres);

        // Update VWAP
        self.bar.update_vwap(mkt_data);

        // Update CVD
        self.calculate_cvd(mkt_data, prev_price);

        // Create new bar
        if self.bar.imb.abs() >= self.bar.imb_thres {
            let completed_bar = self.clone(); // Bar is completed

            // Update historical threshold
            if self.bar.historical_threshold.len() >= self.threshold_count() {
                self.bar.historical_threshold.pop_front();
            }
            self.bar.historical_threshold.push_back(self.bar.imb_thres);

            // Update the parameters and prepare for new bar.
            // These parameters will be used for the NEXT BAR.
            let b_t = self.bar.imb / self.bar.tsize as f32;
            self.bar.ewma_imb_current = b_t * self.bar.ewma_factor
                + (1.0 - self.bar.ewma_factor) * self.bar.ewma_imb_current; // EWMA_t = lambda * IMB_t + (1 - lambda) * EWMA_t-1
            self.bar.ewma_t_current = self.bar.tsize as f32 * self.bar.ewma_factor
                + (1.0 - self.bar.ewma_factor) * self.bar.ewma_t_current; // EWMA_t = lambda * t_t + (1 - lambda) * EWMA_t-1

            // Manually set a threshold's limit to prevent the threshold explosion
            let threshold_candidate = self.bar.ewma_imb_current.abs() * self.bar.ewma_t_current;
            let threshold_max = self
                .bar
                .historical_threshold
                .clone()
                .iter()
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap_or(&threshold_candidate)
                * 1.5;
            let threshold_min = self
                .bar
                .historical_threshold
                .clone()
                .iter()
                .min_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap_or(&threshold_candidate)
                * 0.5;
            self.bar.imb_thres = threshold_candidate.min(threshold_max).max(threshold_min);
            self.bar.candle_opened = false; // Close the candle

            return Some(completed_bar);
        }

        None
    }

    pub fn reset(&mut self) {
        self.bar.reset();
    }
}

impl BarImpl for DollarImbalanceBar {
    fn calculate_imbalance(&self, mkt_data: &MarketData, prev_price: f32) -> f32 {
        let price_change = mkt_data.price - prev_price;
        let tick_imbalance = match price_change.total_cmp(&0.0) {
            std::cmp::Ordering::Greater => 1.0,
            std::cmp::Ordering::Less => -1.0,
            std::cmp::Ordering::Equal => 0.0,
        };

        tick_imbalance * mkt_data.quantity * mkt_data.price
    }

    fn should_update(&self, _mkt_data: &MarketData) -> bool {
        true // Dollar imbalance should always update
    }

    fn threshold_count(&self) -> usize {
        50
    }
}

impl VolumeDelta for DollarImbalanceBar {
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
