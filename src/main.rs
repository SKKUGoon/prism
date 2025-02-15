use crate::data::{
    liquidation::binance_liquidation_future::BinanceFutureLiquidationStreamHandler,
    market::{
        binance_aggtrade_future::BinanceFutureAggTradeStreamHandler,
        binance_aggtrade_spot::BinanceSpotAggTradeStreamHandler,
        upbit_aggtrade_spot::UpbitSpotAggTradeStreamHandler,
    },
    markprice::binance_markprice_future::BinanceFutureMarkPriceStreamHandler,
    orderbook::{
        binance_orderbook_future::BinanceFutureOrderbookStreamHandler,
        binance_orderbook_spot::BinanceSpotOrderbookStreamHandler, book::Orderbook,
        upbit_orderbook_spot::UpbitSpotOrderbookStreamHandler,
    },
    stream::StreamHandler,
};
use channel::{FutureChannel, SpotChannel};
use config::read_env_config;
use database::postgres::timescale_batch_writer;
use log::{error, info, warn};
use prism::{
    executor::{PrismConfig, PrismTradeManager},
    stream::{future::FutureStream, spot::SpotStream, FutureReceivers},
};
use tokio::signal;

mod channel;
mod config;
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
    let env_var = read_env_config();

    /* Create threaded task set */
    let mut tasks = tokio::task::JoinSet::new();

    /* Create channels for thread communication */
    let binance_fut = FutureChannel::new(env_var.channel_capacity);
    let binance_spt = SpotChannel::new(env_var.channel_capacity);
    let upbit_spt = SpotChannel::new(env_var.channel_capacity);

    /*
    Start stream managers.
    Transfer data to tx_fut_exec and tx_spt_exec
    */
    let mut binance_future = FutureStream::new(
        binance_fut.ob.mng.1,
        binance_fut.agg.1,
        FutureReceivers {
            rx_markprice: binance_fut.additional.mark.1,
            rx_liquidation: binance_fut.additional.liq.1,
        },
        binance_fut.exec.0,
    );
    let mut binance_spot =
        SpotStream::new(binance_spt.ob.mng.1, binance_spt.agg.1, binance_spt.exec.0);
    let mut upbit_spot = SpotStream::new(upbit_spt.ob.mng.1, upbit_spt.agg.1, upbit_spt.exec.0);

    /*
    Start Data Manager.
    Receive data from tx_fut_exec and tx_spt_exec.
    */
    let mut core_config = PrismConfig::default();
    core_config.enable_data_dump(env_var.data_dump);

    let mut core_mng = PrismTradeManager::new(
        core_config,
        binance_fut.exec.1,
        binance_spt.exec.1,
        binance_fut.db.0,
        binance_spt.db.0,
    );

    /* Feature Creation Engine Start */
    tasks.spawn(async move { binance_future.work().await });
    tasks.spawn(async move { binance_spot.work().await });
    tasks.spawn(async move { upbit_spot.work().await });
    tasks.spawn(async move { core_mng.work().await });

    /* Timescale Insertion */
    if env_var.data_dump {
        tasks.spawn(async move {
            if let Err(e) =
                timescale_batch_writer("binance", &env_var.table_fut, binance_fut.db.1).await
            {
                error!("Timescale batch writer error: {}", e);
            }
        });

        tasks.spawn(async move {
            if let Err(e) =
                timescale_batch_writer("binance", &env_var.table_spt, binance_spt.db.1).await
            {
                error!("Timescale batch writer error: {}", e);
            }
        });
    }

    /* Start Orderbook Container */
    let mut binance_fbook = Orderbook::new(binance_fut.ob.raw.1, binance_fut.ob.mng.0);
    tasks.spawn(async move { binance_fbook.listen().await });

    let mut binance_sbook = Orderbook::new(binance_spt.ob.raw.1, binance_spt.ob.mng.0);
    tasks.spawn(async move { binance_sbook.listen().await });

    let mut upbit_sbook = Orderbook::new(upbit_spt.ob.raw.1, upbit_spt.ob.mng.0);
    tasks.spawn(async move { upbit_sbook.listen().await });

    /* Start Data Streams */
    let binance_future_aggtrade = BinanceFutureAggTradeStreamHandler::new(
        env_var.symbol_binance_fut.clone(),
        binance_fut.agg.0,
    );
    tasks.spawn(spawn_future_aggtrade_task(binance_future_aggtrade));

    let binance_future_liquidation = BinanceFutureLiquidationStreamHandler::new(
        env_var.symbol_binance_fut.clone(),
        binance_fut.additional.liq.0,
    );
    tasks.spawn(spawn_liquidation_task(binance_future_liquidation));

    let binance_future_markprice = BinanceFutureMarkPriceStreamHandler::new(
        env_var.symbol_binance_fut.clone(),
        binance_fut.additional.mark.0,
    );
    tasks.spawn(spawn_markprice_task(binance_future_markprice));

    let binance_spot_aggtrade = BinanceSpotAggTradeStreamHandler::new(
        env_var.symbol_binance_spt.clone(),
        binance_spt.agg.0,
    );
    tasks.spawn(spawn_spot_aggtrade_task(binance_spot_aggtrade));

    let upbit_spot_aggtrade_krw = UpbitSpotAggTradeStreamHandler::new(
        env_var.symbol_upbit_krw.clone(),
        upbit_spt.agg.0.clone(),
    );
    tasks.spawn(spawn_upbit_spot_aggtrade_task(upbit_spot_aggtrade_krw));

    let upbit_spot_aggtrade_btc = UpbitSpotAggTradeStreamHandler::new(
        env_var.symbol_upbit_btc.clone(),
        upbit_spt.agg.0.clone(),
    );
    tasks.spawn(spawn_upbit_spot_aggtrade_task(upbit_spot_aggtrade_btc));

    let upbit_spot_aggtrade_usdt = UpbitSpotAggTradeStreamHandler::new(
        env_var.symbol_upbit_usdt.clone(),
        upbit_spt.agg.0.clone(),
    );
    tasks.spawn(spawn_upbit_spot_aggtrade_task(upbit_spot_aggtrade_usdt));

    let binance_future_orderbook = BinanceFutureOrderbookStreamHandler::new(
        env_var.symbol_binance_fut.clone(),
        binance_fut.ob.raw.0.clone(),
    );
    tasks.spawn(spawn_future_orderbook_task(binance_future_orderbook));

    let binance_spot_orderbook = BinanceSpotOrderbookStreamHandler::new(
        env_var.symbol_binance_spt.clone(),
        binance_spt.ob.raw.0.clone(),
    );
    tasks.spawn(spawn_spot_orderbook_task(binance_spot_orderbook));

    let upbit_spot_orderbook = UpbitSpotOrderbookStreamHandler::new(
        env_var.symbol_binance_spt.clone(),
        upbit_spt.ob.raw.0.clone(),
    );
    tasks.spawn(spawn_upbit_spot_orderbook_task(upbit_spot_orderbook));

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

async fn spawn_upbit_spot_aggtrade_task(handler: UpbitSpotAggTradeStreamHandler) {
    loop {
        warn!("Attempting to connect to Upbit Aggtrades");
        if let Err(e) = handler.connect().await {
            error!("Upbit Aggtrades connection error: {}", e);
        }
        warn!("Upbit Aggtrades: Retrying in 5 seconds");
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

async fn spawn_upbit_spot_orderbook_task(handler: UpbitSpotOrderbookStreamHandler) {
    loop {
        warn!("Attempting to connect to Upbit Spot");
        if let Err(e) = handler.connect().await {
            error!("Upbit Spot connection error: {}", e);
        }
        warn!("Upbit Spot: Retrying in 5 seconds");
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
