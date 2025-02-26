use crate::data::{
    depth::{
        binance::future::BinanceFutureOrderbookStreamHandler,
        binance::spot::BinanceSpotOrderbookStreamHandler,
    },
    exchanges::{FutureDataChannels, SpotDataChannels},
    liquidation::binance::future::BinanceFutureLiquidationStreamHandler,
    market::{
        binance::future::BinanceFutureAggTradeStreamHandler,
        binance::spot::BinanceSpotAggTradeStreamHandler,
    },
    markprice::binance::future::BinanceFutureMarkPriceStreamHandler,
    stream::StreamHandler,
};
use log::{error, warn};
use tokio::task::JoinSet;

pub struct BinanceThreads {
    future: FutureDataChannels,
    spot: SpotDataChannels,
}

impl BinanceThreads {
    pub fn new(future: FutureDataChannels, spot: SpotDataChannels) -> Self {
        Self { future, spot }
    }

    pub fn spawn_streams(
        self,
        tasks: &mut JoinSet<()>,
        future_symbol: String,
        spot_symbol: String,
    ) {
        if future_symbol == "NO_SYMBOL" && spot_symbol == "NO_SYMBOL" {
            warn!("No symbols specified, skipping Binance streams");
            return;
        }

        // Future Streams
        tasks.spawn(spawn_future_aggtrade_task(
            BinanceFutureAggTradeStreamHandler::new(future_symbol.clone(), self.future.agg_out),
        ));
        tasks.spawn(spawn_future_orderbook_task(
            BinanceFutureOrderbookStreamHandler::new(future_symbol.clone(), self.future.ob_out),
        ));
        tasks.spawn(spawn_future_liquidation_task(
            BinanceFutureLiquidationStreamHandler::new(future_symbol.clone(), self.future.liq_out),
        ));
        tasks.spawn(spawn_future_markprice_task(
            BinanceFutureMarkPriceStreamHandler::new(future_symbol.clone(), self.future.mark_out),
        ));

        // Spot Streams
        tasks.spawn(spawn_spot_aggtrade_task(
            BinanceSpotAggTradeStreamHandler::new(spot_symbol.clone(), self.spot.agg_out),
        ));
        tasks.spawn(spawn_spot_orderbook_task(
            BinanceSpotOrderbookStreamHandler::new(spot_symbol.clone(), self.spot.ob_out),
        ));
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
