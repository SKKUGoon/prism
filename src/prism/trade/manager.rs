use super::{
    input_channel::{BinanceFeatureProcessed, DatabaseRecord, UpbitFeatureProcessed},
    strategy::snipe_large_order::SnipeLargeOrderStrategy,
};
use crate::prism::trade::{intra_exchange_param::IntraParams, TradeConfig};
use tokio::select;

#[allow(dead_code)]
pub struct TradeManager {
    config: TradeConfig,

    // Channels (Input)
    binance: BinanceFeatureProcessed,
    upbit: UpbitFeatureProcessed,

    // Strategy Channels (Output)
    snipe_large_order: DatabaseRecord,

    // Parameters
    binance_futures: IntraParams,
    binance_spot: IntraParams,
    upbit_spot: IntraParams,
}

impl TradeManager {
    pub fn new(
        binance_ch: BinanceFeatureProcessed,
        upbit_ch: UpbitFeatureProcessed,
        snipe_large_order_ch: DatabaseRecord,
        config: TradeConfig,
    ) -> Self {
        Self {
            config,
            binance: binance_ch,
            upbit: upbit_ch,
            snipe_large_order: snipe_large_order_ch,
            binance_futures: IntraParams::new(100),
            binance_spot: IntraParams::new(100),
            upbit_spot: IntraParams::new(100),
        }
    }

    pub async fn work(&mut self) {
        // Setup strategies
        let mut upbit_krw_spot_snipe_large_order =
            SnipeLargeOrderStrategy::new("Upbit Spot".to_string());

        loop {
            select! {
                Some(feature) = self.binance.futures.recv() => {
                    self.binance_futures.update_params(&feature);
                    self.binance_futures.update_bars(&feature);

                    // binance_futures_snipe_large_order.evaluate(&self.binance_futures);
                }
                Some(feature) = self.binance.spot.recv() => {
                    self.binance_spot.update_params(&feature);
                    self.binance_spot.update_bars(&feature);

                    // binance_spot_snipe_large_order.evaluate(&self.binance_spot);
                }
                Some(feature) = self.upbit.krw.recv() => {
                    self.upbit_spot.update_params(&feature);
                    self.upbit_spot.update_bars(&feature);

                    let params = upbit_krw_spot_snipe_large_order.evaluate(&self.upbit_spot);
                    if let Some(params) = params {
                        self.snipe_large_order.strat1.send(params).await.unwrap();
                    }
                }
                Some(feature) = self.upbit.btc.recv() => {
                    self.upbit_spot.update_params(&feature);
                    self.upbit_spot.update_bars(&feature);
                }
                Some(feature) = self.upbit.usdt.recv() => {
                    self.upbit_spot.update_params(&feature);
                    self.upbit_spot.update_bars(&feature);
                }
            }
        }
    }
}
