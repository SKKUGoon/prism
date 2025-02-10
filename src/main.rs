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
use data::{
    liquidation::binance_liquidation_future::BinanceFutureLiquidationStreamHandler,
    markprice::binance_markprice_future::BinanceFutureMarkPriceStreamHandler,
};
use database::postgres::timescale_batch_writer;
use log::{error, info, warn};
use prism::{
    executor::{PrismConfig, PrismTradeManager},
    stream::{future::FutureStreamProcessor, spot::SpotStreamProcessor, FutureReceivers},
};
use std::env;
use tokio::{signal, sync::mpsc};

mod data;
mod database;
mod prism;
mod trade;

/*
(1) Data flows like:
     Websocket -> Data -> Engine -> Executor (or database) -> Trade order
(2) Data from websocket to engine.
     The channel name will be: tx(rx)_(fut/spt)_(ob/agg)_data
     a. (ob)Orderbook
     b. (agg)Aggtrade - AggTrade does not need to be processed by class. So `rx` part goes to straight to engine
(3) Data engineering inside engine. Send to Executor (or database)
     The channel name will be: tx(rx)_(fut/spt)_exec
*/

#[tokio::main]
async fn main() {
    /* Initialize logger */
    env_logger::init();

    /* Get environment variables */
    let data_dump = env::var("DATA_DUMP").unwrap_or_else(|_| "false".to_string()) == "true";
    let symbol_fut = env::var("SYMBOLS_FUT").unwrap_or_else(|_| "xrpusdt".to_string());
    let table_fut = env::var("TABLE_FUT").unwrap_or_else(|_| "feature_xrpusdt_future".to_string());
    let symbol_spt = env::var("SYMBOLS_SPT").unwrap_or_else(|_| "xrpusdt".to_string());
    let table_spt = env::var("TABLE_SPT").unwrap_or_else(|_| "feature_xrpusdt_spot".to_string());
    let channel_capacity: usize = env::var("CHANNEL_CAPACITY")
        .unwrap_or_else(|_| "999".to_string())
        .parse()
        .unwrap_or(999);

    /* Create threaded task set */
    let mut tasks = tokio::task::JoinSet::new();

    /* Create channels for thread communication */
    let (
        // Future: Orderbook -> Orderbook Processor -> Engine
        (tx_fut_ob_raw, rx_fut_ob_raw),
        (tx_fut_ob_mng, rx_fut_ob_mng),
        // Spot: Orderbook -> Orderbook Processor -> Engine
        (tx_spt_ob_raw, rx_spt_ob_raw),
        (tx_spt_ob_mng, rx_spt_ob_mng),
        // Aggtrade -> Engine
        (tx_fut_agg_raw, rx_fut_agg_mng),
        (tx_spt_agg_raw, rx_spt_agg_mng),
        // Mark Price -> Engine
        (tx_fut_mark_raw, rx_fut_mark_mng),
        // Liquidation -> Engine
        (tx_fut_liq_raw, rx_fut_liq_mng),
        // Engine -> Executor
        (tx_fut_exec, rx_fut_exec),
        (tx_spt_exec, rx_spt_exec),
        // Executor -> Database
        (tx_fut_db, rx_fut_db),
        (tx_spt_db, rx_spt_db),
    ) = (
        mpsc::channel(channel_capacity),
        mpsc::channel(channel_capacity),
        mpsc::channel(channel_capacity),
        mpsc::channel(channel_capacity),
        mpsc::channel(channel_capacity),
        mpsc::channel(channel_capacity),
        mpsc::channel(channel_capacity),
        mpsc::channel(channel_capacity),
        mpsc::channel(channel_capacity),
        mpsc::channel(channel_capacity),
        mpsc::channel(channel_capacity),
        mpsc::channel(channel_capacity),
    );

    /*
    Start stream managers.
    Transfer data to tx_fut_exec and tx_spt_exec
    */
    let mut fut_engine = FutureStreamProcessor::new(
        rx_fut_ob_mng,
        rx_fut_agg_mng,
        FutureReceivers {
            rx_markprice: rx_fut_mark_mng,
            rx_liquidation: rx_fut_liq_mng,
        },
        tx_fut_exec,
    );
    let mut spt_engine = SpotStreamProcessor::new(rx_spt_ob_mng, rx_spt_agg_mng, tx_spt_exec);

    /*
    Start Data Manager.
    Receive data from tx_fut_exec and tx_spt_exec.
    */
    let mut core_config = PrismConfig::default();
    core_config.enable_data_dump(data_dump);

    let mut core_mng =
        PrismTradeManager::new(core_config, rx_fut_exec, rx_spt_exec, tx_fut_db, tx_spt_db);

    /* Feature Creation Engine Start */
    tasks.spawn(async move { fut_engine.work().await });
    tasks.spawn(async move { spt_engine.work().await });
    tasks.spawn(async move { core_mng.work().await });

    /* Timescale Insertion */
    if data_dump {
        tasks.spawn(async move {
            if let Err(e) = timescale_batch_writer("binance", &table_fut, rx_fut_db).await {
                error!("Timescale batch writer error: {}", e);
            }
        });

        tasks.spawn(async move {
            if let Err(e) = timescale_batch_writer("binance", &table_spt, rx_spt_db).await {
                error!("Timescale batch writer error: {}", e);
            }
        });
    }

    /* Start Streams */
    let binance_future_aggtrade =
        BinanceFutureAggTradeStreamHandler::new(symbol_fut.clone(), tx_fut_agg_raw);
    tasks.spawn(spawn_future_aggtrade_task(binance_future_aggtrade));

    let binance_future_liquidation =
        BinanceFutureLiquidationStreamHandler::new(symbol_fut.clone(), tx_fut_liq_raw);
    tasks.spawn(spawn_liquidation_task(binance_future_liquidation));

    let binance_future_markprice =
        BinanceFutureMarkPriceStreamHandler::new(symbol_fut.clone(), tx_fut_mark_raw);
    tasks.spawn(spawn_markprice_task(binance_future_markprice));

    let binance_spot_aggtrade =
        BinanceSpotAggTradeStreamHandler::new(symbol_spt.clone(), tx_spt_agg_raw);
    tasks.spawn(spawn_spot_aggtrade_task(binance_spot_aggtrade));

    let mut future_book = Orderbook::new(rx_fut_ob_raw, tx_fut_ob_mng);
    let binance_future_orderbook =
        BinanceFutureOrderbookStreamHandler::new(symbol_fut.clone(), tx_fut_ob_raw.clone());
    tasks.spawn(spawn_future_orderbook_task(binance_future_orderbook));
    tasks.spawn(async move { future_book.listen().await });

    let mut spot_book = Orderbook::new(rx_spt_ob_raw, tx_spt_ob_mng);
    let binance_spot_orderbook =
        BinanceSpotOrderbookStreamHandler::new(symbol_spt.clone(), tx_spt_ob_raw.clone());
    tasks.spawn(spawn_spot_orderbook_task(binance_spot_orderbook));
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

async fn spawn_liquidation_task(handler: BinanceFutureLiquidationStreamHandler) {
    loop {
        warn!("Attempting to connect to Binance Liquidation");
        if let Err(e) = handler.connect().await {
            error!("Binance Liquidation connection error: {}", e);
        }
        warn!("Binance Liquidation: Retrying in 5 seconds");
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
}

async fn spawn_markprice_task(handler: BinanceFutureMarkPriceStreamHandler) {
    loop {
        warn!("Attempting to connect to Binance Mark Price");
        if let Err(e) = handler.connect().await {
            error!("Binance Mark Price connection error: {}", e);
        }
        warn!("Binance Mark Price: Retrying in 5 seconds");
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
}
