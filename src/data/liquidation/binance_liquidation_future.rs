use crate::data::stream::StreamHandler;
use futures::{SinkExt, StreamExt};
use log::{error, info};
use serde::Deserialize;
use std::future::Future;
use tokio::sync::mpsc;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{self, Message},
};

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct BinanceWebsocketFutureLiquidation {
    pub streams: String,
    pub data: FutureLiquidationEvent,
}

#[allow(dead_code, non_snake_case)]
#[derive(Debug, Deserialize)]
pub struct FutureLiquidationEvent {
    pub e: String, // Event Type
    pub E: u64,    // Event Time
    pub o: FutureLiquidationOrder,
}

#[allow(dead_code, non_snake_case)]
#[derive(Debug, Deserialize)]
pub struct FutureLiquidationOrder {
    pub s: String,  // Symbol
    pub S: String,  // Side
    pub o: String,  // Order Type
    pub f: String,  // Time in Force
    pub q: String,  // Original Quantity
    pub p: String,  // Price
    pub ap: String, // Average Price
    pub X: String,  // Order Status
    pub l: String,  // Order Last Filled Quantity
    pub z: String,  // Order Filled Accumulated Quantity
    pub T: u64,     // Order Trade Time
}

pub struct LiquidationData {
    pub side: String,
    pub avg_price: f32,
    pub quantity: f32,
    pub trade_time: u64,
    pub event_time: u64,
}

pub struct BinanceFutureLiquidationStreamHandler {
    pub streams: String,
    pub symbol: String,
    pub tx: mpsc::Sender<LiquidationData>,
}

impl StreamHandler for BinanceFutureLiquidationStreamHandler {
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

            let handler = BinanceFutureLiquidationStreamHandler {
                symbol,
                streams,
                tx,
            };
            handler.handle_liquidation(read, write).await;

            Ok(())
        }))
    }
}

impl BinanceFutureLiquidationStreamHandler {
    pub fn new(symbol: String, tx: mpsc::Sender<LiquidationData>) -> Self {
        Self {
            symbol,
            streams: "forceOrder".to_string(),
            tx,
        }
    }

    pub async fn handle_liquidation<R, S>(&self, mut read: R, mut write: S)
    where
        R: StreamExt<Item = Result<Message, tungstenite::Error>> + Unpin,
        S: SinkExt<Message> + Unpin,
        S::Error: std::fmt::Display,
    {
        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    match serde_json::from_str::<BinanceWebsocketFutureLiquidation>(&text) {
                        Ok(liquidation) => {
                            let update = self.generate_liquidation_update(&liquidation);
                            if self.tx.send(update).await.is_err() {
                                error!("Binance liquidation stream: Failed to send update");
                            }
                        }
                        Err(e) => {
                            error!("Binance liquidation stream: Failed to parse message: {}", e);
                        }
                    }
                }
                Ok(Message::Ping(payload)) => {
                    if let Err(e) = write.send(Message::Pong(payload)).await {
                        error!("Binance liquidation stream: Failed to send pong: {}", e);
                    }
                }
                Ok(Message::Pong(_)) => info!("Binance liquidation stream: Pong received"),
                Ok(Message::Close(_)) => {
                    info!("Binance liquidation stream: Connection closed");
                    break;
                }
                _ => (),
            }
        }
    }

    fn generate_liquidation_update(
        &self,
        update: &BinanceWebsocketFutureLiquidation,
    ) -> LiquidationData {
        LiquidationData {
            side: update.data.o.S.clone(),
            avg_price: update.data.o.ap.parse().unwrap(),
            quantity: update.data.o.q.parse().unwrap(),
            trade_time: update.data.o.T,
            event_time: update.data.E,
        }
    }
}
