use crate::data::{
    exchanges::{FutureDataChannels, SpotDataChannels},
    liquidation::binance_liquidation_future::BinanceFutureLiquidationStreamHandler,
    market::{
        binance_aggtrade_future::BinanceFutureAggTradeStreamHandler,
        binance_aggtrade_spot::BinanceSpotAggTradeStreamHandler,
    },
    markprice::binance_markprice_future::BinanceFutureMarkPriceStreamHandler,
    orderbook::{
        binance_orderbook_future::BinanceFutureOrderbookStreamHandler,
        binance_orderbook_spot::BinanceSpotOrderbookStreamHandler, book::Orderbook,
    },
    stream::StreamHandler,
};
use log::{error, warn};
use tokio::task::JoinSet;

pub struct BinanceStreams {
    future: FutureDataChannels,
    spot: SpotDataChannels,
}

impl BinanceStreams {
    pub fn new(future: FutureDataChannels, spot: SpotDataChannels) -> Self {
        Self { future, spot }
    }

    pub fn spawn_streams(
        self,
        tasks: &mut JoinSet<()>,
        future_symbol: String,
        spot_symbol: String,
    ) {
        let mut binance_fbook = Orderbook::new(self.future.ob_raw_in, self.future.ob_mng_out);
        let mut binance_sbook = Orderbook::new(self.spot.ob_raw_in, self.spot.ob_mng_out);
        tasks.spawn(async move { binance_fbook.listen().await });
        tasks.spawn(async move { binance_sbook.listen().await });

        if future_symbol != "NO_SYMBOL" {
            // User didn't specify a future symbol on purpose
            // Start Streams
            tasks.spawn(spawn_future_aggtrade_task(
                BinanceFutureAggTradeStreamHandler::new(future_symbol.clone(), self.future.agg_out),
            ));
            tasks.spawn(spawn_future_orderbook_task(
                BinanceFutureOrderbookStreamHandler::new(
                    future_symbol.clone(),
                    self.future.ob_raw_out,
                ),
            ));
            tasks.spawn(spawn_future_liquidation_task(
                BinanceFutureLiquidationStreamHandler::new(
                    future_symbol.clone(),
                    self.future.liq_out,
                ),
            ));
            tasks.spawn(spawn_future_markprice_task(
                BinanceFutureMarkPriceStreamHandler::new(
                    future_symbol.clone(),
                    self.future.mark_out,
                ),
            ));
        }

        if spot_symbol != "NO_SYMBOL" {
            // User didn't specify a spot symbol on purpose
            // Start Streams
            tasks.spawn(spawn_spot_aggtrade_task(
                BinanceSpotAggTradeStreamHandler::new(spot_symbol.clone(), self.spot.agg_out),
            ));
            tasks.spawn(spawn_spot_orderbook_task(
                BinanceSpotOrderbookStreamHandler::new(spot_symbol.clone(), self.spot.ob_raw_out),
            ));
        }
    }
}

/* Future Streams */

async fn spawn_future_aggtrade_task(handler: BinanceFutureAggTradeStreamHandler) {
    loop {
        warn!("Attempting to connect to Binance Aggtrades");
        if let Err(e) = handler.connect().await {
            error!("Binance Aggtrades connection error: {}", e);
        }
        warn!("Binance Aggtrades: Retrying in 5 seconds");
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
}

async fn spawn_future_orderbook_task(handler: BinanceFutureOrderbookStreamHandler) {
    loop {
        warn!("Attempting to connect to Binance");
        if let Err(e) = handler.connect().await {
            error!("Binance connection error: {}", e);
        }
        warn!("Binance: Retrying in 5 seconds");
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
}

async fn spawn_future_liquidation_task(handler: BinanceFutureLiquidationStreamHandler) {
    loop {
        warn!("Attempting to connect to Binance Liquidation");
        if let Err(e) = handler.connect().await {
            error!("Binance Liquidation connection error: {}", e);
        }
        warn!("Binance Liquidation: Retrying in 5 seconds");
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
}

async fn spawn_future_markprice_task(handler: BinanceFutureMarkPriceStreamHandler) {
    loop {
        warn!("Attempting to connect to Binance Mark Price");
        if let Err(e) = handler.connect().await {
            error!("Binance Mark Price connection error: {}", e);
        }
        warn!("Binance Mark Price: Retrying in 5 seconds");
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
}

/* Spot Streams */
async fn spawn_spot_aggtrade_task(handler: BinanceSpotAggTradeStreamHandler) {
    loop {
        warn!("Attempting to connect to Binance Aggtrades");
        if let Err(e) = handler.connect().await {
            error!("Binance Aggtrades connection error: {}", e);
        }
        warn!("Binance Aggtrades: Retrying in 5 seconds");
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
}

async fn spawn_spot_orderbook_task(handler: BinanceSpotOrderbookStreamHandler) {
    loop {
        warn!("Attempting to connect to Binance Spot");
        if let Err(e) = handler.connect().await {
            error!("Binance Spot connection error: {}", e);
        }
        warn!("Binance Spot: Retrying in 5 seconds");
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
}
