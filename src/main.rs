use crate::data::orderbook::{
    binance_orderbook_future::BinanceFutureOrderbookStreamHandler, book::Orderbook,
};
use crate::data::stream::StreamHandler;
use data::market::binance_aggtrade_future::BinanceFutureAggTradeStreamHandler;
use data::market::binance_aggtrade_spot::BinanceSpotAggTradeStreamHandler;
use data::orderbook::binance_orderbook_spot::BinanceSpotOrderbookStreamHandler;
use database::postgres::timescale_batch_writer;
use log::{error, info, warn};
use prism::engine::PrismaEngine;
use tokio::{signal, sync::mpsc};

mod data;
mod database;
mod prism;
mod trade;

#[tokio::main]
async fn main() {
    env_logger::init();
    let test_symbol = String::from("xrpusdt");
    let (tx_future_orderbook_data, rx_future_orderbook_data) = mpsc::channel(9999);
    let (tx_future_orderbook_prism, rx_future_orderbook_prism) = mpsc::channel(9999);
    let (tx_spot_orderbook_data, rx_spot_orderbook_data) = mpsc::channel(9999);
    let (tx_spot_orderbook_prism, rx_spot_orderbook_prism) = mpsc::channel(9999);

    let (tx_future_aggtrade_data, rx_future_aggtrade_prism) = mpsc::channel(9999);
    let (tx_spot_aggtrade_data, rx_spot_aggtrade_prism) = mpsc::channel(9999);

    let (tx_future_prisma_feature, rx_future_prisma_feature) = mpsc::channel(2000);
    let (tx_spot_prisma_feature, rx_spot_prisma_feature) = mpsc::channel(2000);

    /* Engine Start */
    let mut future_engine = PrismaEngine::new(
        "future",
        rx_future_orderbook_prism,
        rx_future_aggtrade_prism,
        tx_future_prisma_feature,
    );
    let mut spot_engine = PrismaEngine::new(
        "spot",
        rx_spot_orderbook_prism,
        rx_spot_aggtrade_prism,
        tx_spot_prisma_feature,
    );

    tokio::spawn(async move { future_engine.work().await });
    tokio::spawn(async move { spot_engine.work().await });

    /* Timescale Insertion */
    tokio::spawn(async move {
        if let Err(e) = timescale_batch_writer(rx_future_prisma_feature).await {
            error!("Timescale batch writer error: {}", e);
        }
    });

    tokio::spawn(async move {
        if let Err(e) = timescale_batch_writer(rx_spot_prisma_feature).await {
            error!("Timescale batch writer error: {}", e);
        }
    });

    /* Price Feed */
    // Future
    let binance_future_aggtrade =
        BinanceFutureAggTradeStreamHandler::new(test_symbol.clone(), tx_future_aggtrade_data);

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
        BinanceSpotAggTradeStreamHandler::new(test_symbol.clone(), tx_spot_aggtrade_data);

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
    let mut future_book = Orderbook::new(rx_future_orderbook_data, tx_future_orderbook_prism);
    let binance_future_orderbook = BinanceFutureOrderbookStreamHandler::new(
        test_symbol.clone(),
        tx_future_orderbook_data.clone(),
    );

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
    let mut spot_book = Orderbook::new(rx_spot_orderbook_data, tx_spot_orderbook_prism);
    let binance_spot_orderbook =
        BinanceSpotOrderbookStreamHandler::new(test_symbol.clone(), tx_spot_orderbook_data.clone());

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
