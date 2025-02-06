use crate::data::{orderbook::book::OrderbookUpdateStream, stream::StreamHandler};

use futures::{SinkExt, StreamExt};
use log::{error, info};
use serde::Deserialize;
use std::future::Future;
use tokio::sync::mpsc;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{self, Message},
};

/* Binance Orderbook Snapshot */

#[allow(dead_code, non_snake_case)]
#[derive(Debug, Deserialize)]
pub struct SpotDepthSnapShot {
    pub lastUpdateId: u64,
    pub bids: Vec<(String, String)>,
    pub asks: Vec<(String, String)>,
}

pub async fn fetch_depth_snapshot(symbol: &str) -> Result<SpotDepthSnapShot, reqwest::Error> {
    // https://developers.binance.com/docs/binance-spot-api-docs/rest-api/market-data-endpoints
    let url = format!(
        "https://api.binance.com/api/v3/depth?symbol={}&limit=3000",
        symbol.to_uppercase()
    );

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await?
        .error_for_status()?
        .json::<SpotDepthSnapShot>()
        .await?;

    Ok(response)
}

/* Binance Orderbook Stream */

#[allow(dead_code, non_snake_case)]
#[derive(Debug, Deserialize)]
pub struct SpotDepthEvent {
    pub e: String,                // Event type
    pub E: u64,                   // Event time
    pub s: String,                // Symbol
    pub U: u64,                   // First update ID in event
    pub u: u64,                   // Final update ID in event
    pub b: Vec<(String, String)>, // Bids to update
    pub a: Vec<(String, String)>, // Asks to update
}

pub struct BinanceSpotOrderbookStreamHandler {
    streams: String,
    pub symbol: String,
    pub tx: mpsc::Sender<OrderbookUpdateStream>,
}

impl StreamHandler for BinanceSpotOrderbookStreamHandler {
    fn connect(&self) -> Box<dyn Future<Output = Result<(), tungstenite::Error>> + Send + Unpin> {
        let symbol = self.symbol.clone();
        let streams = self.streams.clone();
        let tx = self.tx.clone();

        Box::new(Box::pin(async move {
            let ws_url = format!("wss://stream.binance.com:443/ws/{}@{}", symbol, streams);
            let (ws_stream, _) = connect_async(&ws_url).await?;
            let (write, read) = ws_stream.split();

            let snapshot = fetch_depth_snapshot(&symbol).await.unwrap();

            let handler = BinanceSpotOrderbookStreamHandler {
                symbol,
                streams,
                tx,
            };
            handler.handle_orderbook(read, write, snapshot).await;

            Ok(())
        }))
    }
}

impl BinanceSpotOrderbookStreamHandler {
    pub fn new(symbol: String, tx: mpsc::Sender<OrderbookUpdateStream>) -> Self {
        Self {
            symbol,
            streams: "depth".to_string(),
            tx,
        }
    }

    pub async fn handle_orderbook<R, S>(
        &self,
        mut read: R,
        mut write: S,
        snapshot: SpotDepthSnapShot,
    ) where
        R: StreamExt<Item = Result<Message, tungstenite::Error>> + Unpin,
        S: SinkExt<Message> + Unpin,
        S::Error: std::fmt::Display,
    {
        let mut last_update_id = snapshot.lastUpdateId;

        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    match serde_json::from_str::<SpotDepthEvent>(&text) {
                        Ok(diff) => {
                            // Snapshot handling
                            if diff.u <= last_update_id || diff.U <= last_update_id {
                                error!(
                                    "Binance orderbook stream: Event out of order - reinitializing"
                                );
                                continue;
                            }

                            // Send processed binance update to tx
                            let update = self.generate_orderbook_update(&diff);
                            if self.tx.send(update).await.is_err() {
                                error!("Binance orderbook stream: Failed to send update")
                            };

                            last_update_id = diff.u;
                        }
                        Err(e) => {
                            error!(
                                "Binance orderbook stream: Failed to parse message: {} {}",
                                e, text
                            );
                        }
                    }
                }
                Ok(Message::Ping(payload)) => {
                    if let Err(e) = write.send(Message::Pong(payload)).await {
                        error!("Binance orderbook stream: Failed to send Pong: {}", e);
                    }
                }
                Ok(Message::Pong(_)) => info!("Binance orderbook stream: Pong received"),
                Ok(Message::Close(_)) => {
                    info!("Binance orderbook stream: Connection closed");
                    break;
                }
                _ => (),
            }
        }
    }

    fn generate_orderbook_update(&self, update: &SpotDepthEvent) -> OrderbookUpdateStream {
        let bids = update
            .b
            .iter()
            .map(|(price, quantity)| (price.to_string(), quantity.to_string()))
            .collect();

        let asks = update
            .a
            .iter()
            .map(|(price, quantity)| (price.to_string(), quantity.to_string()))
            .collect();

        OrderbookUpdateStream {
            bids,
            asks,
            trade_time: update.u,
            event_time: update.E,
            last_update_exchange: "Binance".to_string(),
        }
    }
}
