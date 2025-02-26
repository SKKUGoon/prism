use crate::data::{depth, liquidation, market, markprice};
use crate::prism::core::{Core, MarketState};
use crate::prism::orderbook::Orderbook;
use rust_decimal::prelude::FromStr;
use rust_decimal::Decimal;
use tokio::sync::mpsc;

pub struct FutureCore {
    pub mark: mpsc::Receiver<markprice::MarkPriceData>,
    pub liq: mpsc::Receiver<liquidation::LiquidationData>,
}

impl Core<FutureCore> {
    pub fn new(
        ob: mpsc::Receiver<depth::OrderbookUpdateStream>,
        agg: mpsc::Receiver<market::MarketData>,
        mark: mpsc::Receiver<markprice::MarkPriceData>,
        liq: mpsc::Receiver<liquidation::LiquidationData>,
    ) -> Self {
        Self {
            ob,
            agg,
            additional: FutureCore { mark, liq },
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

                Some(mark) = self.additional.mark.recv() => {
                    self.market_state.event_time = mark.event_time;
                    // Update mark price
                    self.market_state.mark_price = Some(Decimal::from_str(&mark.mark_price).unwrap());
                    self.market_state.index_price = Some(Decimal::from_str(&mark.index_price).unwrap());
                    self.market_state.funding_rate = Some(Decimal::from_str(&mark.funding_rate).unwrap());
                    self.market_state.next_funding_time = Some(mark.next_funding_time);

                    self.debug();
                }

                Some(liq) = self.additional.liq.recv() => {
                    // Update liquidation
                    self.market_state.liq_quantity = Decimal::from_str(&liq.quantity).unwrap();
                    self.market_state.liq_price = Decimal::from_str(&liq.avg_price).unwrap();
                    self.market_state.liq_side = liq.side;

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
