use crate::data::{markprice::MarkPriceData, stream::StreamHandler};
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
pub struct BinanceWebsocketFutureMarkPrice {
    pub stream: String,
    pub data: FutureMarkPriceEvent,
}

#[allow(dead_code, non_snake_case)]
#[derive(Debug, Deserialize)]
pub struct FutureMarkPriceEvent {
    pub e: String, // Event type
    pub E: u64,    // Event time
    pub s: String, // Symbol
    pub p: String, // Mark price
    pub i: String, // Index price
    pub P: String, // Estimated Settle Price, only useful in the last hour before the settlement starts
    pub r: String, // Funding rate
    pub T: u64,    // Next funding time
}

pub struct BinanceFutureMarkPriceStreamHandler {
    pub streams: String,
    pub symbol: String,
    pub tx: mpsc::Sender<MarkPriceData>,
}

impl StreamHandler for BinanceFutureMarkPriceStreamHandler {
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

            let handler = BinanceFutureMarkPriceStreamHandler {
                symbol,
                streams,
                tx,
            };
            handler.handle_markprice(read, write).await;

            Ok(())
        }))
    }
}

impl BinanceFutureMarkPriceStreamHandler {
    pub fn new(symbol: String, tx: mpsc::Sender<MarkPriceData>) -> Self {
        Self {
            symbol,
            streams: "markPrice".to_string(),
            tx,
        }
    }

    pub async fn handle_markprice<R, S>(&self, mut read: R, mut write: S)
    where
        R: StreamExt<Item = Result<Message, tungstenite::Error>> + Unpin,
        S: SinkExt<Message> + Unpin,
        S::Error: std::fmt::Display,
    {
        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    match serde_json::from_str::<BinanceWebsocketFutureMarkPrice>(&text) {
                        Ok(markprice) => {
                            let update = self.generate_markprice_update(&markprice);
                            if self.tx.send(update).await.is_err() {
                                error!("Binance mark price stream: Failed to send update");
                            }
                        }
                        Err(e) => {
                            error!("Binance mark price stream: Failed to parse message: {}", e);
                        }
                    }
                }
                Ok(Message::Ping(payload)) => {
                    if let Err(e) = write.send(Message::Pong(payload)).await {
                        error!("Binance mark price stream: Failed to send pong: {}", e);
                    }
                }
                Ok(Message::Pong(_)) => info!("Binance mark price stream: Pong received"),
                Ok(Message::Close(_)) => {
                    info!("Binance mark price stream: Connection closed");
                    break;
                }
                _ => (),
            }
        }
    }

    fn generate_markprice_update(&self, update: &BinanceWebsocketFutureMarkPrice) -> MarkPriceData {
        MarkPriceData {
            mark_price: update.data.p.clone(),
            index_price: update.data.i.clone(),
            funding_rate: update.data.r.clone(),
            next_funding_time: update.data.T,
            event_time: update.data.E,
        }
    }
}
