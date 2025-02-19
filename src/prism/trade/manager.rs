use crate::prism::trade::{
    binance::BinanceFeatureProcessed, intra_exchange_param::IntraParams,
    upbit::UpbitFeatureProcessed, TradeConfig,
};
use log::info;
use tokio::select;

use super::strategy::snipe_large_order::SnipeLargeOrderStrategy;

pub struct TradeManager {
    config: TradeConfig,

    // Channels
    binance: BinanceFeatureProcessed,
    upbit: UpbitFeatureProcessed,

    // Parameters
    binance_futures: IntraParams,
    binance_spot: IntraParams,
    upbit_spot: IntraParams,
}

impl TradeManager {
    pub fn new(
        binance_ch: BinanceFeatureProcessed,
        upbit_ch: UpbitFeatureProcessed,
        config: TradeConfig,
    ) -> Self {
        Self {
            config,
            binance: binance_ch,
            upbit: upbit_ch,
            binance_futures: IntraParams::new(100),
            binance_spot: IntraParams::new(100),
            upbit_spot: IntraParams::new(100),
        }
    }

    pub async fn work(&mut self) {
        // Setup strategies
        let mut binance_futures_snipe_large_order =
            SnipeLargeOrderStrategy::new("Binance Futures".to_string());
        let mut binance_spot_snipe_large_order =
            SnipeLargeOrderStrategy::new("Binance Spot".to_string());
        let mut upbit_krw_spot_snipe_large_order =
            SnipeLargeOrderStrategy::new("Upbit Spot".to_string());

        loop {
            select! {
                Some(feature) = self.binance.futures.recv() => {
                    self.binance_futures.update_params(&feature);
                    self.binance_futures.update_bars(&feature);

                    binance_futures_snipe_large_order.evaluate(&self.binance_futures);
                }
                Some(feature) = self.binance.spot.recv() => {
                    self.binance_spot.update_params(&feature);
                    self.binance_spot.update_bars(&feature);

                    binance_spot_snipe_large_order.evaluate(&self.binance_spot);
                }
                Some(feature) = self.upbit.krw.recv() => {
                    self.upbit_spot.update_params(&feature);
                    self.upbit_spot.update_bars(&feature);

                    let params = upbit_krw_spot_snipe_large_order.evaluate(&self.upbit_spot);
                    if let Some(params) = params {
                        info!("Upbit KRW Spot Snipe Large Order: {:?}", params);
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
