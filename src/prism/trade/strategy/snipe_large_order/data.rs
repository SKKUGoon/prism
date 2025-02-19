use super::SnipeLargeOrderParams;
use log::info;
use tokio::sync::mpsc;
use tokio_postgres::Client;

const BATCH_SIZE: usize = 100;

pub struct SnipeLargeOrderChannelPairs {
    pub db: (
        mpsc::Sender<SnipeLargeOrderParams>,
        mpsc::Receiver<SnipeLargeOrderParams>,
    ),
}

impl SnipeLargeOrderChannelPairs {
    pub fn new(max_capacity: usize) -> Self {
        let (tx_db, rx_db) = mpsc::channel(max_capacity);
        Self { db: (tx_db, rx_db) }
    }
}

pub async fn record_snipe_large_order_params(
    client: &Client,
    table: &str,
    mut rx: mpsc::Receiver<SnipeLargeOrderParams>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = Vec::new();

    while let Some(feature) = rx.recv().await {
        buffer.push(feature);
        if buffer.len() >= BATCH_SIZE {
            batch_insert(client, table, &buffer).await?;
            info!("Inserted {} features into {:?}", buffer.len(), table);
            buffer.clear();
        }
    }

    Ok(())
}

async fn batch_insert(
    client: &Client,
    table: &str,
    features: &Vec<SnipeLargeOrderParams>,
) -> Result<(), Box<dyn std::error::Error>> {
    if features.is_empty() {
        return Ok(());
    }

    let base_query = format!(
        "INSERT INTO {} (
            time, price, best_bid_price, best_bid_volume, best_ask_price, best_ask_volume, best_bid_ask_spread,
            total_bid_volume, total_ask_volume, market_buy_size, market_sell_size
        ) VALUES ",
        table,
    );

    let mut placeholders: Vec<String> = Vec::new();
    let mut values: Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>> = Vec::new();

    let mut param_index = 1;
    for feature in features {
        placeholders.push(format!(
            "(to_timestamp(${}::FLOAT8), ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${})",
            param_index,
            param_index + 1,
            param_index + 2,
            param_index + 3,
            param_index + 4,
            param_index + 5,
            param_index + 6,
            param_index + 7,
            param_index + 8,
            param_index + 9,
            param_index + 10,
        ));

        values.push(Box::new(feature.time as f64 / 1000.0));
        values.push(Box::new(feature.price));
        values.push(Box::new(feature.best_bid_price));
        values.push(Box::new(feature.best_bid_volume));
        values.push(Box::new(feature.best_ask_price));
        values.push(Box::new(feature.best_ask_volume));
        values.push(Box::new(feature.best_bid_ask_spread));
        values.push(Box::new(feature.total_bid_volume));
        values.push(Box::new(feature.total_ask_volume));
        values.push(Box::new(feature.market_buy_size));
        values.push(Box::new(feature.market_sell_size));

        param_index += 11;
    }

    let query = format!("{}{}", base_query, placeholders.join(", "));
    client
        .execute(
            &query,
            &values
                .iter()
                .map(|p| p.as_ref() as &(dyn tokio_postgres::types::ToSql + Sync))
                .collect::<Vec<_>>(),
        )
        .await?;

    Ok(())
}
