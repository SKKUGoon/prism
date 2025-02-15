pub mod binance_aggtrade_future;
pub mod binance_aggtrade_spot;
pub mod upbit_aggtrade_spot;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MarketData {
    pub price: f32,
    pub quantity: f32,
    pub buyer_market_maker: bool,
    pub trade_time: u64,
    pub event_time: u64,
}
