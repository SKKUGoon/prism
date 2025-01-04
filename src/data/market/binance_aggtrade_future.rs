use futures::{SinkExt, StreamExt};
use log::{error, info};
use serde::Deserialize;
use std::future::Future;
use tokio::sync::mpsc;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{self, Message},
};

use crate::data::stream::StreamHandler;

/* Binance AggTrade Stream */

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct BinanceWebsocketFutureAggTrade {
    pub stream: String,
    pub data: FutureAggTradeEvent,
}

#[allow(dead_code, non_snake_case)]
#[derive(Debug, Deserialize)]
pub struct FutureAggTradeEvent {
    pub e: String, // Event type
    pub E: u64,    // Event time
    pub s: String, // Symbol
    pub a: u64,    // Aggregate trade ID
    pub p: String, // Price
    pub q: String, // Quantity
    pub f: u64,    // First trade ID
    pub l: u64,    // Last trade ID
    pub T: u64,    // Trade time
    pub m: bool,   // Is the buyer the market maker?
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MarketData {
    pub price: f32,
    pub quantity: f32,
    pub buyer_market_maker: bool,
    pub time: u64,
}

pub struct BinanceFutureAggTradeStreamHandler {
    pub streams: String,
    pub symbol: String,
    pub tx: mpsc::Sender<MarketData>,
}

impl StreamHandler for BinanceFutureAggTradeStreamHandler {
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

            let handler = BinanceFutureAggTradeStreamHandler {
                symbol,
                streams,
                tx,
            };
            handler.handle_aggtrade(read, write).await;

            Ok(())
        }))
    }
}

impl BinanceFutureAggTradeStreamHandler {
    pub fn new(symbol: String, tx: mpsc::Sender<MarketData>) -> Self {
        Self {
            symbol,
            streams: "aggTrade".to_string(),
            tx,
        }
    }

    pub async fn handle_aggtrade<R, S>(&self, mut read: R, mut write: S)
    where
        R: StreamExt<Item = Result<Message, tungstenite::Error>> + Unpin,
        S: SinkExt<Message> + Unpin,
        S::Error: std::fmt::Display,
    {
        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    match serde_json::from_str::<BinanceWebsocketFutureAggTrade>(&text) {
                        Ok(aggtrade) => {
                            let update = self.generate_aggtrade_update(&aggtrade);
                            if self.tx.send(update).await.is_err() {
                                error!("Binance aggtrade stream: Failed to send update");
                            }
                        }
                        Err(e) => {
                            error!("Binance aggtrade stream: Failed to parse message: {}", e);
                        }
                    }
                }
                Ok(Message::Ping(payload)) => {
                    if let Err(e) = write.send(Message::Pong(payload)).await {
                        error!("Binance aggtrade stream: Failed to send Pong: {}", e);
                    }
                }
                Ok(Message::Pong(_)) => info!("Binance aggtrade stream: Pong received"),
                Ok(Message::Close(_)) => {
                    info!("Binance aggtrade stream: Connection closed");
                    break;
                }
                _ => (),
            }
        }
    }

    fn generate_aggtrade_update(&self, update: &BinanceWebsocketFutureAggTrade) -> MarketData {
        MarketData {
            price: update.data.p.parse().unwrap(),
            quantity: update.data.q.parse().unwrap(),
            buyer_market_maker: update.data.m,
            time: update.data.T,
        }
    }
}
