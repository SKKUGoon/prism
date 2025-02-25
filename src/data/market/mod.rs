pub mod binance;
pub mod upbit;

// Output data format
#[derive(Debug, Clone)]
pub struct MarketData {
    pub price: f32,
    pub quantity: f32,
    pub buyer_market_maker: bool,
    pub trade_time: u64,
    pub event_time: u64,
}
