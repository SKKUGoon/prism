use crate::data::{
    exchanges::SpotDataChannels,
    market::upbit::spot::UpbitSpotAggTradeStreamHandler,
    orderbook::{upbit::spot::UpbitSpotOrderbookStreamHandler, Orderbook},
    stream::StreamHandler,
};
use log::{error, info, warn};
use tokio::task::JoinSet;

pub struct UpbitStreams {
    spot: SpotDataChannels,
}

impl UpbitStreams {
    pub fn new(spot: SpotDataChannels) -> Self {
        Self { spot }
    }

    pub fn spawn_streams(self, tasks: &mut JoinSet<()>, symbols: String) {
        if symbols == "NO_SYMBOL" {
            warn!("No symbols specified, skipping Upbit streams");
            return;
        }

        let mut upbit_sbook = Orderbook::new(self.spot.ob_raw_in, self.spot.ob_mng_out);
        tasks.spawn(async move { upbit_sbook.listen().await });

        // Spot Streams
        info!("Starting Upbit Streams for {}", symbols);
        tasks.spawn(spawn_spot_aggtrade_task(
            UpbitSpotAggTradeStreamHandler::new(symbols.clone(), self.spot.agg_out),
        ));
        tasks.spawn(spawn_spot_orderbook_task(
            UpbitSpotOrderbookStreamHandler::new(symbols.clone(), self.spot.ob_raw_out),
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
