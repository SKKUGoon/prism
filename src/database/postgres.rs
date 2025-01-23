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
    #[allow(unused_variables)] schema: &str,
    #[allow(unused_variables)] table: &str,
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
        "INSERT INTO {}.{} (
            time, source, price, maker_quantity, taker_quantity, 
            obi, obi_005p, obi_01p, obi_02p, obi_05p, 
            tib_id, tib_imb, tib_thres, 
            vmb_id, vmb_imb, vmb_thres, 
            vmm_id, vmm_imb, vmm_thres, 
            vmt_id, vmt_imb, vmt_thres, 
            dib_id, dib_imb, dib_thres
        ) VALUES ",
        schema, table,
    );

    let mut placeholders: Vec<String> = Vec::new();
    let mut values: Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>> = Vec::new();

    let mut param_index = 1;
    for feature in features {
        placeholders.push(format!(
            "(to_timestamp(${}::FLOAT8), ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${})",
            param_index,      // time
            param_index + 1,  // source
            param_index + 2,  // price
            param_index + 3,  // maker_quantity
            param_index + 4,  // taker_quantity
            param_index + 5,  // obi
            param_index + 6,  // obi_005p
            param_index + 7,  // obi_01p
            param_index + 8,  // obi_02p
            param_index + 9,  // obi_05p
            // Tick imbalance bar
            param_index + 10, // id (tib_id)
            param_index + 11, // imb (tib_imb)
            param_index + 12, // thres (tib_thres)
            // Volume imbalance bar
            param_index + 13, // id (vmb_id)
            param_index + 14, // imb (vmb_imb)
            param_index + 15, // thres (vmb_thres)
            // Volume imbalance bar maker
            param_index + 16, // id (vmm_id)
            param_index + 17, // imb (vmm_imb)
            param_index + 18, // thres (vmm_thres)
            // Volume imbalance bar taker
            param_index + 19, // id (vmt_id)
            param_index + 20, // imb (vmt_imb)
            param_index + 21, // thres (vmt_thres)
            // Dollar imbalance bar
            param_index + 22, // id (dib_id)
            param_index + 23, // imb (dib_imb)
            param_index + 24, // thres (dib_thres)
        ));

        values.push(Box::new(feature.feature_time as f64 / 1000.0)); // time
        values.push(Box::new(feature.source.clone())); // source
        values.push(Box::new(feature.price)); // price
        values.push(Box::new(feature.maker_quantity)); // maker_quantity
        values.push(Box::new(feature.taker_quantity)); // taker_quantity
        values.push(Box::new(feature.obi)); // obi
        values.push(Box::new(feature.obi_range.0)); // obi_005p
        values.push(Box::new(feature.obi_range.1)); // obi_01p
        values.push(Box::new(feature.obi_range.2)); // obi_02p
        values.push(Box::new(feature.obi_range.3)); // obi_05p

        // Tick imbalance bar
        values.push(Box::new(feature.tick_imbalance_bar.id.clone())); // tib_id
        values.push(Box::new(feature.tick_imbalance_bar.imb)); // tib_imb
        values.push(Box::new(feature.tick_imbalance_bar.imb_thres)); // tib_thres

        // Volume imbalance bar
        values.push(Box::new(feature.volume_imbalance_bar_both.id.clone())); // vmb_id
        values.push(Box::new(feature.volume_imbalance_bar_both.imb)); // vmb_imb
        values.push(Box::new(feature.volume_imbalance_bar_both.imb_thres)); // vmb_thres

        // Volume imbalance bar maker
        values.push(Box::new(feature.volume_imbalance_bar_maker.id.clone())); // vmm_id
        values.push(Box::new(feature.volume_imbalance_bar_maker.imb)); // vmm_imb
        values.push(Box::new(feature.volume_imbalance_bar_maker.imb_thres)); // vmm_thres

        // Volume imbalance bar taker
        values.push(Box::new(feature.volume_imbalance_bar_taker.id.clone())); // vmt_id
        values.push(Box::new(feature.volume_imbalance_bar_taker.imb)); // vmt_imb
        values.push(Box::new(feature.volume_imbalance_bar_taker.imb_thres)); // vmt_thres

        // Dollar imbalance bar
        values.push(Box::new(feature.dollar_imbalance_bar_both.id.clone())); // dib_id
        values.push(Box::new(feature.dollar_imbalance_bar_both.imb)); // dib_imb
        values.push(Box::new(feature.dollar_imbalance_bar_both.imb_thres)); // dib_thres

        param_index += 25;
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
