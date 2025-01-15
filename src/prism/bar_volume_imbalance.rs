use crate::data::market::binance_aggtrade_future::MarketData;

#[derive(Debug, Clone)]
pub struct VolumeImbalanceBar {
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
    pub volume_type: VolumeType,

    // Constant
    genesis_collect_period: u64, // Cumulative time for creating the first bar
    ewma_factor: f32,

    // EWMA - No Reset
    ewma_imb_current: f32,
    ewma_t_current: f32,
}

#[derive(Debug, Clone)]
pub enum VolumeType {
    Maker,
    Taker,
    Both,
}

impl VolumeImbalanceBar {
    pub fn new(volume_type: VolumeType) -> Self {
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

            genesis_collect_period: 5000000, // 5 seconds
            ewma_factor: 0.9, // Higher factor = more weights to recent data, more responsive to volatile market
            volume_type,
            ewma_imb_current: 0.0,
            ewma_t_current: 0.0,
        }
    }

    pub fn genesis_bar(&mut self, mkt_data: &MarketData) -> Option<VolumeImbalanceBar> {
        match self.ts {
            Some(ts) => {
                // Retrieve the most recent price from `pe`
                let prev_price = self.pc.unwrap(); // Guaranteed to be Some
                let price_change = mkt_data.price - prev_price;

                let tick_imbalance = if price_change > 0.0 {
                    1.0 // Price Increase - Motivated by the buyers
                } else if price_change < 0.0 {
                    -1.0 // Price Decrease - Motivated by the sellers
                } else {
                    0.0 // Price Stable - Order Match
                };

                match self.volume_type {
                    VolumeType::Maker => {
                        if mkt_data.buyer_market_maker {
                            self.imb += tick_imbalance * mkt_data.quantity;
                        }
                    }
                    VolumeType::Taker => {
                        if !mkt_data.buyer_market_maker {
                            self.imb += tick_imbalance * mkt_data.quantity;
                        }
                    }
                    VolumeType::Both => {
                        self.imb += tick_imbalance * mkt_data.quantity;
                    }
                }
                self.tsize += 1;

                // Update existing bar
                self.te = Some(mkt_data.time);
                self.pc = Some(mkt_data.price);
                self.ph = Some(self.ph.unwrap().max(mkt_data.price));
                self.pl = Some(self.pl.unwrap().min(mkt_data.price));

                if let Some(te) = self.te {
                    if te - ts >= self.genesis_collect_period {
                        // Genesis bar creation is done after pre-adjusted amount of time
                        if te - ts >= self.genesis_collect_period {
                            // Create new bar
                            self.ewma_imb_current = self.imb * self.ewma_factor
                                + (1.0 - self.ewma_factor) * self.ewma_imb_current; // EWMA_t = lambda * IMB_t + (1 - lambda) * EWMA_t-1
                            self.ewma_t_current = self.tsize as f32 * self.ewma_factor
                                + (1.0 - self.ewma_factor) * self.ewma_t_current; // EWMA_t = lambda * t_t + (1 - lambda) * EWMA_t-1

                            return Some(self.clone());
                        }
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

    pub fn bar(&mut self, mkt_data: &MarketData) -> Option<VolumeImbalanceBar> {
        match self.ts {
            Some(_) => {
                let prev_price = self.pc.unwrap(); // Guaranteed to be Some
                let price_change = mkt_data.price - prev_price;

                let tick_imbalance = if price_change > 0.0 {
                    1.0 // Price Increase - Motivated by the buyers
                } else if price_change < 0.0 {
                    -1.0 // Price Decrease - Motivated by the sellers
                } else {
                    0.0 // Price Stable - Order Match
                };

                match self.volume_type {
                    VolumeType::Maker => {
                        if mkt_data.buyer_market_maker {
                            self.imb += tick_imbalance * mkt_data.quantity;
                        }
                    }
                    VolumeType::Taker => {
                        if !mkt_data.buyer_market_maker {
                            self.imb += tick_imbalance * mkt_data.quantity;
                        }
                    }
                    VolumeType::Both => {
                        self.imb += tick_imbalance * mkt_data.quantity;
                    }
                }
                self.tsize += 1;

                // Update existing bar
                self.te = Some(mkt_data.time);
                self.pc = Some(mkt_data.price);
                self.ph = Some(self.ph.unwrap().max(mkt_data.price));
                self.pl = Some(self.pl.unwrap().min(mkt_data.price));

                if self.imb.abs() >= self.ewma_imb_current * self.ewma_t_current {
                    // Record new EWMA
                    self.ewma_imb_current = self.imb * self.ewma_factor
                        + (1.0 - self.ewma_factor) * self.ewma_imb_current; // EWMA_t = lambda * IMB_t + (1 - lambda) * EWMA_t-1
                    self.ewma_t_current = self.tsize as f32 * self.ewma_factor
                        + (1.0 - self.ewma_factor) * self.ewma_t_current; // EWMA_t = lambda * t_t + (1 - lambda) * EWMA_t-1

                    // Create new bar
                    let bar = self.clone();
                    return Some(bar);
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
