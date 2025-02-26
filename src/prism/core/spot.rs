use crate::data::{depth, market};
use crate::prism::core::{Core, MarketState};
use crate::prism::orderbook::Orderbook;
use rust_decimal::prelude::FromStr;
use rust_decimal::Decimal;
use tokio::sync::mpsc;

pub struct SpotCore;

impl Core<SpotCore> {
    pub fn new(
        ob: mpsc::Receiver<depth::OrderbookUpdateStream>,
        agg: mpsc::Receiver<market::MarketData>,
    ) -> Self {
        Self {
            ob,
            agg,
            additional: SpotCore,
            market_state: MarketState::new(),
            total_orderbook: Orderbook::new(),
        }
    }

    pub async fn work(&mut self) {
        loop {
            tokio::select! {
                Some(market) = self.agg.recv() => {
                    // Update price
                    self.market_state.event_time = market.event_time;
                    self.market_state.transaction_time = market.trade_time;
                    self.market_state.price = Decimal::from_str(&market.price).unwrap();

                    match market.buyer_market_maker {
                        true => {
                            self.market_state.sell_quantity = Decimal::from_str(&market.quantity).unwrap();
                            self.market_state.buy_quantity = Decimal::from(0);
                        }
                        false => {
                            self.market_state.buy_quantity = Decimal::from_str(&market.quantity).unwrap();
                            self.market_state.sell_quantity = Decimal::from(0);
                        }
                    }
                    self.debug();
                }

                Some(ob) = self.ob.recv() => {
                    // Update orderbook
                    self.total_orderbook.update(&ob).await;

                    self.debug();
                }

                else => {}
            }
        }
    }
}
