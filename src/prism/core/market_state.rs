use rust_decimal::Decimal;

#[derive(Debug)]
pub struct MarketState {
    // Time: Websocket Received Time
    pub event_time: u64,
    // Time: Transaction Time
    pub transaction_time: u64,
    // Price
    pub price: Decimal,
    pub index_price: Option<Decimal>,
    pub vwap: Option<Decimal>,
    // Quantity
    pub sell_quantity: Decimal,
    pub buy_quantity: Decimal,
    // Mark Price
    pub mark_price: Option<Decimal>,
    pub funding_rate: Option<Decimal>,
    pub next_funding_time: Option<u64>,
    // Liquidation
    pub liq_quantity: Decimal,
    pub liq_price: Decimal,
    pub liq_side: String,
}

impl MarketState {
    pub fn new() -> Self {
        Self {
            event_time: 0,
            transaction_time: 0,
            price: Decimal::from(0),
            index_price: None,
            vwap: None,
            sell_quantity: Decimal::from(0),
            buy_quantity: Decimal::from(0),
            mark_price: None,
            funding_rate: None,
            next_funding_time: None,
            liq_quantity: Decimal::from(0),
            liq_price: Decimal::from(0),
            liq_side: String::from(""),
        }
    }
}
