use crate::data::market::MarketData;
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

/* Binance AggTrade Stream */

#[allow(dead_code, non_snake_case)]
#[derive(Debug, Deserialize)]
pub struct SpotAggTradeEvent {
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

pub struct BinanceSpotAggTradeStreamHandler {
    pub stream: String,
    pub symbol: String,
    pub tx: mpsc::Sender<MarketData>,
}

impl StreamHandler for BinanceSpotAggTradeStreamHandler {
    fn connect(&self) -> Box<dyn Future<Output = Result<(), tungstenite::Error>> + Send + Unpin> {
        let symbol = self.symbol.clone();
        let stream = self.stream.clone();
        let tx = self.tx.clone();

        Box::new(Box::pin(async move {
            let ws_url = format!("wss://stream.binance.com:443/ws/{}@aggTrade", symbol);
            let (ws_stream, _) = connect_async(ws_url).await?;
            let (write, read) = ws_stream.split();

            let handler = BinanceSpotAggTradeStreamHandler { symbol, stream, tx };
            handler.handle_aggtrade(read, write).await;

            Ok(())
        }))
    }
}

impl BinanceSpotAggTradeStreamHandler {
    pub fn new(symbol: String, tx: mpsc::Sender<MarketData>) -> Self {
        Self {
            symbol,
            stream: "aggTrade".to_string(),
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
                Ok(Message::Text(text)) => match serde_json::from_str::<SpotAggTradeEvent>(&text) {
                    Ok(aggtrade) => {
                        let update = self.generate_aggtrade_update(&aggtrade);
                        if self.tx.send(update).await.is_err() {
                            error!("Binance aggtrade stream: Failed to send update");
                        }
                    }
                    Err(e) => {
                        error!("Binance aggtrade stream: Failed to parse message: {}", e);
                    }
                },
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

    fn generate_aggtrade_update(&self, update: &SpotAggTradeEvent) -> MarketData {
        MarketData {
            price: update.p.clone(),
            quantity: update.q.clone(),
            buyer_market_maker: update.m,
            trade_time: update.T,
            event_time: update.E,
        }
    }
}
