pub mod binance;
pub mod manager;
pub mod param;
pub mod upbit;

pub struct TradeConfig {
    leverage: u8,
    max_leverage: u8,
    loss_cut: Option<f32>,
    take_profit: Option<f32>,
    data_dump: bool,
}

impl TradeConfig {
    pub fn default() -> Self {
        Self {
            leverage: 1,
            max_leverage: 30,
            loss_cut: Some(0.05),
            take_profit: Some(0.10),
            data_dump: false,
        }
    }

    #[allow(dead_code)]
    pub fn enable_data_dump(&mut self, data_dump: bool) {
        self.data_dump = data_dump;
    }
}

// Trading fees are different for each exchange
// trade_fees: 0.0004, // 0.04% for market takers
