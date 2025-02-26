use crate::data::{
    depth::upbit::spot::UpbitSpotOrderbookStreamHandler, exchanges::SpotDataChannels,
    market::upbit::spot::UpbitSpotAggTradeStreamHandler, stream::StreamHandler,
};
use log::{error, info, warn};
use tokio::task::JoinSet;

pub struct UpbitThreads {
    spot: SpotDataChannels,
}

impl UpbitThreads {
    pub fn new(spot: SpotDataChannels) -> Self {
        Self { spot }
    }

    pub fn spawn_streams(self, tasks: &mut JoinSet<()>, symbols: String) {
        if symbols == "NO_SYMBOL" {
            warn!("No symbols specified, skipping Upbit streams");
            return;
        }

        // Spot Streams
        info!("Starting Upbit Streams for {}", symbols);
        tasks.spawn(spawn_spot_aggtrade_task(
            UpbitSpotAggTradeStreamHandler::new(symbols.clone(), self.spot.agg_out),
        ));
        tasks.spawn(spawn_spot_orderbook_task(
            UpbitSpotOrderbookStreamHandler::new(symbols.clone(), self.spot.ob_out),
        ));
    }
}

async fn spawn_spot_aggtrade_task(handler: UpbitSpotAggTradeStreamHandler) {
    loop {
        warn!("Attempting to connect to Upbit Aggtrades");
        if let Err(e) = handler.connect().await {
            error!("Upbit Aggtrades connection error: {}", e);
        }
        warn!("Upbit Aggtrades: Retrying in 5 seconds");
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
}

async fn spawn_spot_orderbook_task(handler: UpbitSpotOrderbookStreamHandler) {
    loop {
        warn!("Attempting to connect to Upbit Spot");
        if let Err(e) = handler.connect().await {
            error!("Upbit Spot connection error: {}", e);
        }
        warn!("Upbit Spot: Retrying in 5 seconds");
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
}
