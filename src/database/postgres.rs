use log::{error, info};
use tokio::sync::mpsc;
use tokio_postgres::{Client, NoTls};

use crate::prism::engine::PrismaFeature;

#[allow(dead_code)]
pub async fn connect_to_timescale() -> Result<Client, Box<dyn std::error::Error>> {
    let connection_str =
        "host=host.docker.internal port=10501 user=postgres password=postgres dbname=postgres";

    let (client, connection) = tokio_postgres::connect(connection_str, NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            error!("Timescale connection error: {}", e);
        }
    });

    Ok(client)
}

pub async fn timescale_batch_writer(
    schema: &str,
    table: &str,
    mut rx: mpsc::Receiver<PrismaFeature>,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = connect_to_timescale().await?;

    let mut buffer = Vec::new();

    while let Some(feature) = rx.recv().await {
        buffer.push(feature);
        if buffer.len() >= 100 {
            batch_insert_into_timescale(schema, table, &client, &buffer).await?;
            info!(
                "Inserted {} features into {:?}.{:?}",
                buffer.len(),
                schema,
                table
            );
            buffer.clear();
        }
    }

    Ok(())
}

async fn batch_insert_into_timescale(
    schema: &str,
    table: &str,
    client: &Client,
    features: &Vec<PrismaFeature>,
) -> Result<(), Box<dyn std::error::Error>> {
    if features.is_empty() {
        return Ok(());
    }

    let base_query = format!(
        "INSERT INTO {}.{} (time, source, price, maker_quantity, taker_quantity, obi, obi_005p, obi_01p, obi_02p, obi_05p, tib_id, tib_ts, tib_te, tib_ps, tib_pe, tib_imb) VALUES ",
        schema,
        table,
    );

    let mut placeholders: Vec<String> = Vec::new();
    let mut values: Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>> = Vec::new();

    let mut param_index = 1;
    for feature in features {
        placeholders.push(format!(
            "(to_timestamp(${}::FLOAT8), ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, to_timestamp(${}::FLOAT8), to_timestamp(${}::FLOAT8), ${}, ${}, ${})",
            param_index, // time
            param_index + 1, // source
            param_index + 2, // price
            param_index + 3, // maker_quantity
            param_index + 4, // taker_quantity
            param_index + 5, // obi
            param_index + 6, // obi_005p
            param_index + 7, // obi_01p
            param_index + 8, // obi_02p
            param_index + 9, // obi_05p
            param_index + 10, // tib_id
            param_index + 11, // tib_ts
            param_index + 12, // tib_te
            param_index + 13, // tib_ps
            param_index + 14, // tib_pe
            param_index + 15, // tib_imb
        ));
        values.push(Box::new(feature.feature_time as f64 / 1000.0));
        values.push(Box::new(feature.source.clone()));
        values.push(Box::new(feature.price));
        values.push(Box::new(feature.maker_quantity));
        values.push(Box::new(feature.taker_quantity));
        values.push(Box::new(feature.obi));
        values.push(Box::new(feature.obi_range.0));
        values.push(Box::new(feature.obi_range.1));
        values.push(Box::new(feature.obi_range.2));
        values.push(Box::new(feature.obi_range.3));
        values.push(Box::new(feature.tib.id.clone()));
        values.push(Box::new(feature.tib.ts as f64 / 1000.0));
        values.push(Box::new(feature.tib.te as f64 / 1000.0));
        values.push(Box::new(feature.tib.ps.unwrap_or(0.0)));
        values.push(Box::new(feature.tib.pe.unwrap_or(0.0)));
        values.push(Box::new(feature.tib.imb));

        param_index += 16;
    }

    let combined_data = placeholders.join(",");
    let query = format!("{}{}", base_query, combined_data);
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
