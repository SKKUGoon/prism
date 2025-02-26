pub mod binance;

#[derive(Debug)]
pub struct LiquidationData {
    pub side: String,
    pub avg_price: String,
    pub quantity: String,
    pub trade_time: u64,
    pub event_time: u64,
}
