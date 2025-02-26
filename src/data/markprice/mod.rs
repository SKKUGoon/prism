pub mod binance;

#[derive(Debug)]
pub struct MarkPriceData {
    pub mark_price: String,
    pub index_price: String,
    pub funding_rate: String,
    pub next_funding_time: u64,
    pub event_time: u64,
}
