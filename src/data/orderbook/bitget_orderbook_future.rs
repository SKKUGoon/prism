use crate::data::{orderbook::book::OrderbookUpdateStream, stream::StreamHandler};
use futures::{SinkExt, StreamExt};
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::future::Future;
use tokio::sync::mpsc;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{self, Message},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct BitgetStreamArg {
    pub instType: String, // e.g., "USDT-FUTURES"
    pub channel: String,  // e.g., "books"
    pub instId: String,   // e.g., "BTCUSDT"
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct BitgetStreamConfirm {
    pub event: String,
    pub arg: BitgetStreamArg,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct BitgetDepthMessage {
    pub action: String, // "snapshot" or "update"
    pub arg: BitgetStreamArg,
    pub data: Vec<DepthData>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
#[allow(dead_code)]
pub struct DepthData {
    pub asks: Vec<(String, String)>, // List of asks (price, quantity)
    pub bids: Vec<(String, String)>, // List of bids (price, quantity)
    pub checksum: i64,               // Checksum for validation
    pub ts: String,                  // Timestamp as a string
}

pub struct BitgetFutureOrderbookStreamHandler {
    pub symbol: String,
    pub tx: mpsc::Sender<OrderbookUpdateStream>,
}

impl StreamHandler for BitgetFutureOrderbookStreamHandler {
    fn connect(&self) -> Box<dyn Future<Output = Result<(), tungstenite::Error>> + Send + Unpin> {
        let symbol = self.symbol.clone();
        let tx = self.tx.clone();

        Box::new(Box::pin(async move {
            let ws_url = "wss://ws.bitget.com/v2/ws/public";
            let (ws_stream, _) = connect_async(ws_url).await?;
            let (mut write, read) = ws_stream.split();

            let subscription = serde_json::json!({
                "op": "subscribe",
                "args": [BitgetStreamArg{
                    instType: "USDT-FUTURES".to_string(),
                    channel: "books".to_string(),
                    instId: symbol.clone()
                }]
            })
            .to_string();

            write.send(Message::Text(subscription.into())).await?;

            // Create a new handler instance for the async block
            let handler = BitgetFutureOrderbookStreamHandler { symbol, tx };
            handler.handle_orderbook(read, write).await;

            Ok(())
        }))
    }
}

#[allow(dead_code)]
impl BitgetFutureOrderbookStreamHandler {
    pub fn new(symbol: String, tx: mpsc::Sender<OrderbookUpdateStream>) -> Self {
        Self { symbol, tx }
    }

    pub async fn handle_orderbook<R, S>(&self, mut read: R, mut write: S)
    where
        R: StreamExt<Item = Result<Message, tungstenite::Error>> + Unpin,
        S: SinkExt<Message> + Unpin,
        S::Error: std::fmt::Display,
    {
        let mut last_update_id = 0;
        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    match serde_json::from_str::<BitgetDepthMessage>(&text) {
                        Ok(msg) => match msg.action.as_str() {
                            "snapshot" => {
                                last_update_id = msg.data[0].ts.clone().parse::<u64>().unwrap();
                                let update = self.generate_orderbook_update(&msg);
                                if self.tx.send(update).await.is_err() {
                                    error!("Failed to send update");
                                }
                            }
                            "update" => {
                                if msg.data[0].ts.clone().parse::<u64>().unwrap() <= last_update_id
                                {
                                    error!("Event out of order - reinitializing");
                                    continue;
                                }
                                let update = self.generate_orderbook_update(&msg);
                                if self.tx.send(update).await.is_err() {
                                    error!("Failed to send update");
                                }
                                last_update_id = msg.data[0].ts.clone().parse::<u64>().unwrap();
                            }
                            _ => error!("Unknown action {}", msg.action.clone()),
                        },
                        Err(_) => match serde_json::from_str::<BitgetStreamConfirm>(&text) {
                            Ok(_) => {
                                info!("Successfully subscribed");
                            }
                            Err(e) => {
                                error!("Failed to parse message: {}", e);
                            }
                        },
                    }
                }
                Ok(Message::Ping(payload)) => {
                    if let Err(e) = write.send(Message::Pong(payload)).await {
                        error!("Failed to send Pong: {}", e);
                    }
                }
                Ok(Message::Pong(_)) => info!("Pong received"),
                Ok(Message::Close(_)) => {
                    info!("Connection closed");
                    break;
                }
                _ => (),
            }
        }
    }

    fn generate_orderbook_update(&self, update: &BitgetDepthMessage) -> OrderbookUpdateStream {
        let bids = update.data[0]
            .bids
            .iter()
            .map(|(price, quantity)| (price.to_string(), quantity.to_string()))
            .collect();

        let asks = update.data[0]
            .asks
            .iter()
            .map(|(price, quantity)| (price.to_string(), quantity.to_string()))
            .collect();

        OrderbookUpdateStream {
            bids,
            asks,
            trade_time: update.data[0].ts.parse::<u64>().unwrap(),
            event_time: update.data[0].ts.parse::<u64>().unwrap(),
            last_update_exchange: "bitget".to_string(),
        }
    }
}
