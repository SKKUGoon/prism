pub mod binance;
pub mod upbit;

#[derive(Debug, Clone)]
pub struct MarketData {
    pub price: String,
    pub quantity: String,
    pub buyer_market_maker: bool, // true: SELL ORDER, false: BUY ORDER
    pub trade_time: u64,
    pub event_time: u64,
}

// Why buyer_market_maker is true: SELL ORDER, false: BUY ORDER
// Market Maker generally puts limit order
// If the market maker becomes the buyer, it means that the limit order is filled on the bidding side.
// Which means that the market maker has bought the asset that is being sold.
