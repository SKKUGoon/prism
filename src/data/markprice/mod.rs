pub mod binance_markprice_future;

#[allow(dead_code)]
#[derive(Debug)]
pub struct MarkPriceData {
    pub mark_price: f32,
    pub index_price: f32,
    pub funding_rate: f32,
    pub next_funding_time: u64,
    pub event_time: u64,
}
