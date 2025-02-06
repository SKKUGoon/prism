use crate::data::{
    market::{
        binance_aggtrade_future::BinanceFutureAggTradeStreamHandler,
        binance_aggtrade_spot::BinanceSpotAggTradeStreamHandler,
    },
    orderbook::{
        binance_orderbook_future::BinanceFutureOrderbookStreamHandler,
        binance_orderbook_spot::BinanceSpotOrderbookStreamHandler, book::Orderbook,
    },
    stream::StreamHandler,
};
// use database::postgres::timescale_batch_writer;
use log::{error, info, warn};
use prism::{
    executor::{PrismConfig, PrismTradeManager},
    stream_process::StreamProcessor,
    AssetSource,
};
use std::env;
use tokio::{signal, sync::mpsc};

mod data;
mod database;
mod prism;
mod trade;

#[tokio::main]
async fn main() {
    env_logger::init();
    let symbol = env::var("SYMBOLS").unwrap_or_else(|_| "xrpusdt".to_string());
    // let table_fut = env::var("TABLE_FUT").unwrap_or_else(|_| "feature_xrpusdt_future".to_string());
    // let table_spt = env::var("TABLE_SPT").unwrap_or_else(|_| "feature_xrpusdt_spot".to_string());

    /*
    Create Channels

    (1) Data flows like so:
         Websocket -> Data -> Engine -> Executor (or database) -> Trade order
    (2) Data from websocket to engine.
         The channel name will be: tx(rx)_(fut/spt)_(ob/agg)_data
         a. (ob)Orderbook
         b. (agg)Aggtrade - AggTrade does not need to be processed by class. So `rx` part goes to straight to engine
    (3) Data engineering inside engine. Send to Executor (or database)
         The channel name will be: tx(rx)_(fut/spt)_exec
    */

    // Data -> Websocket -> Engine
    const CHANNEL_CAPACITY: usize = 999; // Channel capacity for all channels

    // Future market channels
    let (
        (tx_fut_ob_data, rx_fut_ob_data),
        (tx_fut_ob_prism, rx_fut_ob_prism),
        (tx_fut_agg_data, rx_fut_agg_prism),
        (tx_fut_exec, rx_fut_exec),
        (tx_fut_db, rx_fut_db),
    ) = (
        mpsc::channel(CHANNEL_CAPACITY),
        mpsc::channel(CHANNEL_CAPACITY),
        mpsc::channel(CHANNEL_CAPACITY),
        mpsc::channel(CHANNEL_CAPACITY),
        mpsc::channel(CHANNEL_CAPACITY),
    );

    // Spot market channels
    let (
        (tx_spt_ob_data, rx_spt_ob_data),
        (tx_spt_ob_prism, rx_spt_ob_prism),
        (tx_spt_agg_data, rx_spt_agg_prism),
        (tx_spt_exec, rx_spt_exec),
        (tx_spt_db, rx_spt_db),
    ) = (
        mpsc::channel(CHANNEL_CAPACITY),
        mpsc::channel(CHANNEL_CAPACITY),
        mpsc::channel(CHANNEL_CAPACITY),
        mpsc::channel(CHANNEL_CAPACITY),
        mpsc::channel(CHANNEL_CAPACITY),
    );

    /* Feature Creation Engine Start */
    let mut fut_engine = StreamProcessor::new(
        AssetSource::Future,
        rx_fut_ob_prism,
        rx_fut_agg_prism,
        tx_fut_exec,
    );
    let mut spt_engine = StreamProcessor::new(
        AssetSource::Spot,
        rx_spt_ob_prism,
        rx_spt_agg_prism,
        tx_spt_exec,
    );

    /* Trade Engine Start */
    // let mut core_config = PrismConfig::default();
    // core_config.enable_data_dump();

    let core_config = PrismConfig::default();

    let mut core =
        PrismTradeManager::new(core_config, rx_fut_exec, rx_spt_exec, tx_fut_db, tx_spt_db);

    // Create a JoinSet to manage all spawned tasks
    let mut tasks = tokio::task::JoinSet::new();

    /* Feature Creation Engine Start */
    tasks.spawn(async move { fut_engine.work().await });
    tasks.spawn(async move { spt_engine.work().await });
    tasks.spawn(async move { core.work().await });

    /* Timescale Insertion */
    // tasks.spawn(async move {
    //     if let Err(e) = timescale_batch_writer("binance", &table_fut, rx_fut_db).await {
    //         error!("Timescale batch writer error: {}", e);
    //     }
    // });

    // tasks.spawn(async move {
    //     if let Err(e) = timescale_batch_writer("binance", &table_spt, rx_spt_db).await {
    //         error!("Timescale batch writer error: {}", e);
    //     }
    // });

    /* Price Feed */
    // Future
    let binance_future_aggtrade =
        BinanceFutureAggTradeStreamHandler::new(symbol.clone(), tx_fut_agg_data);

    tasks.spawn(spawn_future_aggtrade_task(binance_future_aggtrade));

    // Spot
    let binance_spot_aggtrade =
        BinanceSpotAggTradeStreamHandler::new(symbol.clone(), tx_spt_agg_data);

    tasks.spawn(spawn_spot_aggtrade_task(binance_spot_aggtrade));

    /* Orderbook */
    // Future
    let mut future_book = Orderbook::new(rx_fut_ob_data, tx_fut_ob_prism);
    let binance_future_orderbook =
        BinanceFutureOrderbookStreamHandler::new(symbol.clone(), tx_fut_ob_data.clone());

    tasks.spawn(spawn_future_orderbook_task(binance_future_orderbook));

    // Spot
    let mut spot_book = Orderbook::new(rx_spt_ob_data, tx_spt_ob_prism);
    let binance_spot_orderbook =
        BinanceSpotOrderbookStreamHandler::new(symbol.clone(), tx_spt_ob_data.clone());

    tasks.spawn(spawn_spot_orderbook_task(binance_spot_orderbook));

    tasks.spawn(async move { future_book.listen().await });

    tasks.spawn(async move { spot_book.listen().await });

    /* Graceful Shutdown */
    tokio::select! {
        _ = signal::ctrl_c() => {
            info!("Received Ctrl+C, shutting down...");
        }
        Some(res) = tasks.join_next() => {
            match res {
                Ok(_) => warn!("A task completed unexpectedly"),
                Err(e) => error!("A task failed: {}", e),
            }
        }
    }

    // Abort all remaining tasks
    tasks.abort_all();
    while tasks.join_next().await.is_some() {
        // Wait for all tasks to complete or be aborted
    }
    info!("Shutdown complete");
}

// Helper functions to create the reconnection tasks
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
