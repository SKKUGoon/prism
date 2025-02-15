pub mod binance_liquidation_future;

#[allow(dead_code)]
#[derive(Debug)]
pub struct LiquidationData {
    pub side: String,
    pub avg_price: f32,
    pub quantity: f32,
    pub trade_time: u64,
    pub event_time: u64,
}
