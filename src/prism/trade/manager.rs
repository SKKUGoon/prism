use crate::prism::{
    stream::FeatureProcessed,
    trade::{
        binance::BinanceFeatureProcessed, param::Params, upbit::UpbitFeatureProcessed, TradeConfig,
    },
};
use log::info;
use tokio::select;
// use tokio::sync::mpsc;

pub struct TradeManager {
    config: TradeConfig,

    // Channels
    binance: BinanceFeatureProcessed,
    upbit: UpbitFeatureProcessed,

    // Parameters
    binance_futures: Params,
    binance_spot: Params,
    upbit_spot: Params,
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
            binance_futures: Params::new(100),
            binance_spot: Params::new(100),
            upbit_spot: Params::new(100),
        }
    }

    pub async fn work(&mut self) {
        loop {
            select! {
                Some(feature) = self.binance.futures.recv() => {
                    // info!("Binance Futures Feature: {:?}", feature);
                }
                Some(feature) = self.binance.spot.recv() => {
                    // info!("Binance Spot Feature: {:?}", feature);
                }
                Some(feature) = self.upbit.krw.recv() => {
                    // info!("Upbit KRW Feature: {:?}", feature);
                }
                Some(feature) = self.upbit.btc.recv() => {
                    info!("Upbit BTC Feature: {:?}", feature);
                }
                Some(feature) = self.upbit.usdt.recv() => {
                    info!("Upbit USDT Feature: {:?}", feature);
                }
            }
        }
    }
}
