use log::error;
// use tokio::sync::mpsc;
use tokio_postgres::{Client, NoTls};

// use crate::orderbook::book::OrderbookUpdateStream;

// pub enum RawData {
//     Orderbook(OrderbookUpdateStream),
// }

#[allow(dead_code)]
pub async fn connect_to_timescale() -> Result<Client, Box<dyn std::error::Error>> {
    let connection_str =
        "host=localhost port=10501 user=postgres password=postgres dbname=postgres";

    let (client, connection) = tokio_postgres::connect(connection_str, NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            error!("Timescale connection error: {}", e);
        }
    });

    Ok(client)
}

// pub async fn timescale_writer(mut rx: mpsc::Receiver<RawData>, client: Client) {
//     while let Some(data) = rx.recv().await {
//         match data {
//             RawData::Orderbook(orderbook) => {}
//         }
//     }
// }
