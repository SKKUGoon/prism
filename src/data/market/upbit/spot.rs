use crate::data::market::MarketData;
use crate::data::stream::StreamHandler;
use futures::{SinkExt, StreamExt};
use log::{error, info};
use serde::Deserialize;
use std::future::Future;
use tokio::sync::mpsc;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{self, Bytes, Message},
};

/* Upbit AggTrade(Trade) Stream */

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct UpbitWebsocketSpotAggTrade {
    #[serde(rename = "type")]
    pub type_fields: String, // trade
    pub code: String,         // Currency pair
    pub timestamp: u64,       // When the message was sent
    pub trade_date: String,   // YYYY-MM-DD
    pub trade_time: String,   // HH:MM:SS
    pub trade_timestamp: u64, // When the trade was executed
    pub trade_price: f32,
    pub trade_volume: f32,
    pub ask_bid: String,
    pub stream_type: String,
}

pub struct UpbitSpotAggTradeStreamHandler {
    pub streams: String,
    pub symbol: String,
    pub tx: mpsc::Sender<MarketData>,
}

impl StreamHandler for UpbitSpotAggTradeStreamHandler {
    fn connect(&self) -> Box<dyn Future<Output = Result<(), tungstenite::Error>> + Send + Unpin> {
        let symbol = self.symbol.clone();
        let tx = self.tx.clone();

        Box::new(Box::pin(async move {
            let ws_url = "wss://api.upbit.com/websocket/v1".to_string();
            let (ws_stream, _) = connect_async(ws_url).await?;
            let (write, read) = ws_stream.split();

            let handler = UpbitSpotAggTradeStreamHandler {
                symbol,
                streams: "trade".to_string(),
                tx,
            };

            handler.handle_aggtrade(read, write).await;

            Ok(())
        }))
    }
}

impl UpbitSpotAggTradeStreamHandler {
    pub fn new(symbol: String, tx: mpsc::Sender<MarketData>) -> Self {
        Self {
            symbol,
            streams: "trade".to_string(),
            tx,
        }
    }

    pub async fn handle_aggtrade<R, S>(&self, mut read: R, mut write: S)
    where
        R: StreamExt<Item = Result<Message, tungstenite::Error>> + Unpin,
        S: SinkExt<Message> + Unpin,
        S::Error: std::fmt::Display,
    {
        // Send subscription message
        let subscribe_msg = format!(
            r#"[{{"ticket":"UNIQUE_TICKET"}},{{"type":{},"codes":["{}"]}}]"#,
            self.streams, self.symbol
        );

        if let Err(e) = write.send(Message::Text(subscribe_msg.into())).await {
            error!("Failed to send subscription message: {}", e);
            return;
        }

        // Upbit requires a ping every 60 seconds
        // It will terminate the connection if no data transmission is detected for 120 seconds
        // https://docs.upbit.com/reference/connection
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    if let Err(e) = write.send(Message::Ping(Bytes::from_static(&[]))).await {
                        error!("Failed to send ping: {}", e);
                        break;
                    }
                }
                msg = read.next() => {
                    match msg {
                        Some(Ok(Message::Binary(binary))) => {
                            match serde_json::from_slice::<UpbitWebsocketSpotAggTrade>(&binary) {
                                Ok(trade) => {
                                    let update = self.generate_aggtrade_update(&trade);
                                    if let Err(e) = self.tx.send(update).await {
                                        error!("Failed to send trade update: {}", e);
                                    }
                                }
                                Err(e) => error!("Failed to parse binary trade data: {}", e),
                            }
                        }
                        Some(Ok(Message::Ping(payload))) => {
                            if let Err(e) = write.send(Message::Pong(payload)).await {
                                error!("Failed to send Pong: {}", e);
                            }
                        }
                        Some(Ok(Message::Pong(_))) => info!("Upbit aggtrade stream: Pong received"),
                        Some(Ok(_)) => {} // Ignore other message types
                        Some(Err(e)) => {
                            error!("WebSocket error: {}", e);
                            break;
                        }
                        None => break,
                    }
                }
            }
        }
    }

    fn generate_aggtrade_update(&self, update: &UpbitWebsocketSpotAggTrade) -> MarketData {
        MarketData {
            price: update.trade_price,
            quantity: update.trade_volume,
            buyer_market_maker: update.ask_bid == "ask",
            trade_time: update.trade_timestamp,
            event_time: update.timestamp,
        }
    }
}
