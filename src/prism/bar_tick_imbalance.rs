use crate::data::market::binance_aggtrade_future::MarketData;

#[derive(Debug, Clone)]
pub struct Tib {
    // Tick Imbalance Bar
    thres: f32, // Threshold for tick imbalance. Cross the threshold, generate a bar
    pub id: String,
    pub ts: u64,         // Time start, Timestamp
    pub te: u64,         // Time end, Timestamp
    pub ps: Option<f32>, // Price start
    pub pe: Option<f32>, // Price end
    pub imb: f32,        // Tick imbalance
}

#[allow(dead_code)]
impl Tib {
    pub fn new(thres: f32) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            thres,
            ts: 0,
            te: 0,
            ps: None,
            pe: None,
            imb: 0.0,
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
        let tick_imbalance = if price_change > 0.0 {
            1.0
        } else if price_change < 0.0 {
            -1.0
        } else {
            0.0
        };

        self.imb += tick_imbalance;

        if self.imb.abs() > self.thres {
            let complete_bar = Tib {
                id: self.id.clone(),
                thres: self.thres,
                ts: self.ts,
                te: mkt_data.time,
                ps: self.ps,
                pe: Some(mkt_data.price),
                imb: self.imb,
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
        self.id = uuid::Uuid::new_v4().to_string();
        self.ts = 0;
        self.te = 0;
        self.ps = None;
        self.pe = None;
        self.imb = 0.0;
    }

    pub fn row(&self) -> Result<(u64, u64, f32, f32, f32), String> {
        // Generate data row. Including time start, time end, price start, price end and tick imbalance
        // For database insertion purpose only.
        match (self.ps, self.pe) {
            (Some(ps), Some(pe)) => {
                let row = (self.ts, self.te, ps, pe, self.imb);
                Ok(row)
            }
            _ => Err("Price start or price end is None".to_string()),
        }
    }
}
