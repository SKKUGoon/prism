use std::env;

use crate::data::orderbook::{
    binance_orderbook_future::BinanceFutureOrderbookStreamHandler, book::Orderbook,
};
use crate::data::stream::StreamHandler;
use data::market::binance_aggtrade_future::BinanceFutureAggTradeStreamHandler;
use data::market::binance_aggtrade_spot::BinanceSpotAggTradeStreamHandler;
use data::orderbook::binance_orderbook_spot::BinanceSpotOrderbookStreamHandler;
use database::postgres::timescale_batch_writer;
use log::{error, info, warn};
use prism::engine::{PrismFeatureEngine, PrismaSource};
use prism::executor::{Prism, PrismConfig};
use tokio::{signal, sync::mpsc};

mod data;
mod database;
mod prism;
mod trade;

#[tokio::main]
async fn main() {
    env_logger::init();
    let symbol = env::var("SYMBOLS").unwrap_or_else(|_| "xrpusdt".to_string());
    let table_fut = format!(
        "{}_future",
        env::var("TABLE").unwrap_or_else(|_| "feature_xrpusdt".to_string())
    );
    let table_spt = format!(
        "{}_spot",
        env::var("TABLE").unwrap_or_else(|_| "feature_xrpusdt".to_string())
    );

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
    let (tx_fut_ob_data, rx_fut_ob_data) = mpsc::channel(999);
    let (tx_fut_ob_prism, rx_fut_ob_prism) = mpsc::channel(999);

    let (tx_spt_ob_data, rx_spt_ob_data) = mpsc::channel(999);
    let (tx_spt_ob_prism, rx_spt_ob_prism) = mpsc::channel(999);

    let (tx_fut_agg_data, rx_fut_agg_prism) = mpsc::channel(999);
    let (tx_spt_agg_data, rx_spt_agg_prism) = mpsc::channel(999);

    // Engine -> Executor (or database)
    let (tx_fut_exec, rx_fut_exec) = mpsc::channel(999);
    let (tx_spt_exec, rx_spt_exec) = mpsc::channel(999);

    let (tx_fut_db, rx_fut_db) = mpsc::channel(999);
    let (tx_spt_db, rx_spt_db) = mpsc::channel(999);

    /* Feature Creation Engine Start */
    let mut fut_engine = PrismFeatureEngine::new(
        PrismaSource::Future,
        rx_fut_ob_prism,
        rx_fut_agg_prism,
        tx_fut_exec,
    );
    let mut spt_engine = PrismFeatureEngine::new(
        PrismaSource::Spot,
        rx_spt_ob_prism,
        rx_spt_agg_prism,
        tx_spt_exec,
    );

    /* Trade Engine Start */
    let mut trade_engine = Prism::new(
        PrismConfig::default(),
        rx_fut_exec,
        rx_spt_exec,
        tx_fut_db,
        tx_spt_db,
    );

    tokio::spawn(async move { fut_engine.work().await });
    tokio::spawn(async move { spt_engine.work().await });
    tokio::spawn(async move { trade_engine.work().await });

    /* Timescale Insertion */
    tokio::spawn(async move {
        if let Err(e) = timescale_batch_writer("binance", &table_fut, rx_fut_db).await {
            error!("Timescale batch writer error: {}", e);
        }
    });

    tokio::spawn(async move {
        if let Err(e) = timescale_batch_writer("binance", &table_spt, rx_spt_db).await {
            error!("Timescale batch writer error: {}", e);
        }
    });

    /* Price Feed */
    // Future
    let binance_future_aggtrade =
        BinanceFutureAggTradeStreamHandler::new(symbol.clone(), tx_fut_agg_data);

    tokio::spawn(async move {
        loop {
            warn!("Attempting to connect to Binance Aggtrades");
            let future = binance_future_aggtrade.connect();
            match future.await {
                Ok(()) => (),
                Err(e) => error!("Binance Aggtrades connection error: {}", e),
            }
            warn!("Binance Aggtrades: Retrying in 5 seconds");
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }
    });

    // Spot
    let binance_spot_aggtrade =
        BinanceSpotAggTradeStreamHandler::new(symbol.clone(), tx_spt_agg_data);

    tokio::spawn(async move {
        loop {
            warn!("Attempting to connect to Binance Aggtrades");
            let future = binance_spot_aggtrade.connect();
            match future.await {
                Ok(()) => (),
                Err(e) => error!("Binance Aggtrades connection error: {}", e),
            }
            warn!("Binance Aggtrades: Retrying in 5 seconds");
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }
    });

    /* Orderbook */
    // Future
    let mut future_book = Orderbook::new(rx_fut_ob_data, tx_fut_ob_prism);
    let binance_future_orderbook =
        BinanceFutureOrderbookStreamHandler::new(symbol.clone(), tx_fut_ob_data.clone());

    tokio::spawn(async move {
        loop {
            warn!("Attempting to connect to Binance");
            let future = binance_future_orderbook.connect();
            match future.await {
                Ok(()) => (),
                Err(e) => error!("Binance connection error: {}", e),
            }
            warn!("Binance: Retrying in 5 seconds");
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }
    });

    tokio::spawn(async move { future_book.listen().await });

    // Spot
    let mut spot_book = Orderbook::new(rx_spt_ob_data, tx_spt_ob_prism);
    let binance_spot_orderbook =
        BinanceSpotOrderbookStreamHandler::new(symbol.clone(), tx_spt_ob_data.clone());

    tokio::spawn(async move {
        loop {
            warn!("Attempting to connect to Binance Spot");
            let future = binance_spot_orderbook.connect();
            match future.await {
                Ok(()) => (),
                Err(e) => error!("Binance Spot connection error: {}", e),
            }
            warn!("Binance Spot: Retrying in 5 seconds");
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }
    });

    tokio::spawn(async move { spot_book.listen().await });

    /* Keep alive */
    tokio::select! {
        // Keep alive
        _ = signal::ctrl_c() => {
            info!("Received Ctrl+C, shutting down...");
        }
    }
}
