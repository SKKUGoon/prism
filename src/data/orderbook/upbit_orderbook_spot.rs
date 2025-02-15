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

/* Upbit Orderbook Stream */

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct UpbitWebsocketSpotOrderbook {
    #[serde(rename = "type")]
    pub type_fields: String, // orderbook
    pub code: String,        // Currency pair
    pub timestamp: u64,      // When the message was sent
    pub total_ask_size: f32, // Total ask size
    pub total_bid_size: f32, // Total bid size
    pub orderbook_units: Vec<OrderbookUnit>,
    pub stream_type: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct OrderbookUnit {
    pub ask_price: f32,
    pub ask_size: f32,
    pub bid_price: f32,
    pub bid_size: f32,
}

pub struct UpbitSpotOrderbookStreamHandler {
    streams: String,
    pub symbol: String,
    pub tx: mpsc::Sender<OrderbookUpdateStream>,
}

impl StreamHandler for UpbitSpotOrderbookStreamHandler {
    fn connect(&self) -> Box<dyn Future<Output = Result<(), tungstenite::Error>> + Send + Unpin> {
        let symbol = self.symbol.clone();
        let tx = self.tx.clone();

        Box::new(Box::pin(async move {
            let ws_url = "wss://api.upbit.com/websocket/v1".to_string();
            let (ws_stream, _) = connect_async(ws_url).await?;
            let (write, read) = ws_stream.split();

            let handler = UpbitSpotOrderbookStreamHandler {
                symbol,
                streams: "orderbook".to_string(),
                tx,
            };

            handler.handle_orderbook(read, write).await;

            Ok(())
        }))
    }
}

impl UpbitSpotOrderbookStreamHandler {
    pub fn new(symbol: String, tx: mpsc::Sender<OrderbookUpdateStream>) -> Self {
        Self {
            symbol,
            streams: "orderbook".to_string(),
            tx,
        }
    }

    pub async fn handle_orderbook<R, S>(&self, mut read: R, mut write: S)
    where
        R: StreamExt<Item = Result<Message, tungstenite::Error>> + Unpin,
        S: SinkExt<Message> + Unpin,
        S::Error: std::fmt::Display,
    {
        // Send subscribe message
        let subscribe_message = format!(
            r#"[{{"ticket":"UNIQUE_TICKET"}},{{"type":{},"codes":["{}"]}}]"#,
            self.streams, self.symbol
        );

        if let Err(e) = write.send(Message::Text(subscribe_message.into())).await {
            error!("Failed to send subscription message: {}", e);
            return;
        }

        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Binary(binary)) => {
                    match serde_json::from_slice::<UpbitWebsocketSpotOrderbook>(&binary) {
                        Ok(orderbook) => {
                            // Snapshot handling
                            if orderbook.stream_type == "SNAPSHOT" {
                                // Send processed upbit update to tx
                                let update = self.generate_orderbook_update(&orderbook);
                                if let Err(e) = self.tx.send(update).await {
                                    error!("Failed to send orderbook update: {}", e);
                                }
                            } else {
                                // Real time
                            }
                        }
                        Err(e) => error!("Failed to parse binary orderbook data: {}", e),
                    }
                }
                Ok(Message::Ping(payload)) => {
                    if let Err(e) = write.send(Message::Pong(payload)).await {
                        error!("Failed to send Pong: {}", e);
                    }
                }
                Ok(Message::Pong(_)) => info!("Upbit orderbook stream: Pong received"),
                Ok(Message::Close(_)) => {
                    info!("Upbit orderbook stream: Connection closed");
                    break;
                }
                _ => (), // Ignore other message types
            }
        }
    }

    fn generate_orderbook_update(
        &self,
        update: &UpbitWebsocketSpotOrderbook,
    ) -> OrderbookUpdateStream {
        let bids = update
            .orderbook_units
            .iter()
            .map(|unit| (unit.bid_price.to_string(), unit.bid_size.to_string()))
            .collect();

        let asks = update
            .orderbook_units
            .iter()
            .map(|unit| (unit.ask_price.to_string(), unit.ask_size.to_string()))
            .collect();

        OrderbookUpdateStream {
            bids,
            asks,
            trade_time: update.timestamp,
            event_time: update.timestamp,
            last_update_exchange: "Upbit".to_string(),
        }
    }
}
