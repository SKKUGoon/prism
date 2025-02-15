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
pub struct FutureDepthSnapShot {
    pub lastUpdateId: u64,
    pub E: u64, // Message output time
    pub T: u64, // Transaction time
    pub bids: Vec<(String, String)>,
    pub asks: Vec<(String, String)>,
}

#[allow(dead_code)]
pub async fn fetch_depth_snapshot(symbol: &str) -> Result<FutureDepthSnapShot, reqwest::Error> {
    // https://developers.binance.com/docs/derivatives/usds-margined-futures/market-data/rest-api/Order-Book
    let url = format!(
        "https://fapi.binance.com/fapi/v1/depth?symbol={}&limit=1000", // 1000 is the max limit. Weight is 20
        symbol
    );

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await?
        .error_for_status()?
        .json::<FutureDepthSnapShot>()
        .await?;

    Ok(response)
}

/* Binance Orderbook Stream */

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct BinanceWebsocketFutureDiffBook {
    pub stream: String,
    pub data: FutureDepthEvent,
}

#[allow(dead_code, non_snake_case)]
#[derive(Debug, Deserialize)]
pub struct FutureDepthEvent {
    pub e: String,                // Event type
    pub E: u64,                   // Event time
    pub T: u64,                   // Transaction time
    pub s: String,                // Symbol
    pub U: u64,                   // First update ID in event
    pub u: u64,                   // Final update ID in event
    pub pu: u64,                  // Final update ID from previous event
    pub b: Vec<(String, String)>, // Bids to update
    pub a: Vec<(String, String)>, // Asks to update
}

pub struct BinanceFutureOrderbookStreamHandler {
    streams: String,
    pub symbol: String,
    pub tx: mpsc::Sender<OrderbookUpdateStream>,
}

impl StreamHandler for BinanceFutureOrderbookStreamHandler {
    fn connect(&self) -> Box<dyn Future<Output = Result<(), tungstenite::Error>> + Send + Unpin> {
        let symbol = self.symbol.clone();
        let streams = self.streams.clone();
        let tx = self.tx.clone();

        Box::new(Box::pin(async move {
            let ws_url = format!(
                "wss://fstream.binance.com/stream?streams={}@{}",
                symbol, streams
            );
            let (ws_stream, _) = connect_async(&ws_url).await?;
            let (write, read) = ws_stream.split();

            let snapshot = fetch_depth_snapshot(&symbol).await.unwrap();

            // Create a new handler instance for the async block
            let handler = BinanceFutureOrderbookStreamHandler {
                symbol,
                streams,
                tx,
            };
            handler.handle_orderbook(read, write, snapshot).await;

            Ok(())
        }))
    }
}

impl BinanceFutureOrderbookStreamHandler {
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
        snapshot: FutureDepthSnapShot,
    ) where
        R: StreamExt<Item = Result<Message, tungstenite::Error>> + Unpin,
        S: SinkExt<Message> + Unpin,
        S::Error: std::fmt::Display,
    {
        let mut last_update_id = snapshot.lastUpdateId;

        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    match serde_json::from_str::<BinanceWebsocketFutureDiffBook>(&text) {
                        Ok(diff) => {
                            // Snapshot handling
                            if diff.data.u <= last_update_id || diff.data.U <= last_update_id {
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

                            last_update_id = diff.data.u;
                        }
                        Err(e) => {
                            error!("Binance orderbook stream: Failed to parse message: {}", e)
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

    fn generate_orderbook_update(
        &self,
        update: &BinanceWebsocketFutureDiffBook,
    ) -> OrderbookUpdateStream {
        let bids = update
            .data
            .b
            .iter()
            .map(|(price, quantity)| (price.to_string(), quantity.to_string()))
            .collect();

        let asks = update
            .data
            .a
            .iter()
            .map(|(price, quantity)| (price.to_string(), quantity.to_string()))
            .collect();

        OrderbookUpdateStream {
            bids,
            asks,
            trade_time: update.data.T,
            event_time: update.data.E,
            last_update_exchange: "Binance".to_string(),
        }
    }
}
