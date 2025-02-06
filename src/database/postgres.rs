use log::{error, info};
use tokio::sync::mpsc;
use tokio_postgres::{Client, NoTls};

use crate::prism::stream_process::FeatureProcessed;

const BATCH_SIZE: usize = 50;

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
    mut rx: mpsc::Receiver<FeatureProcessed>,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = connect_to_timescale().await?;

    let mut buffer = Vec::new();

    while let Some(feature) = rx.recv().await {
        buffer.push(feature);
        if buffer.len() >= BATCH_SIZE {
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
    features: &Vec<FeatureProcessed>,
) -> Result<(), Box<dyn std::error::Error>> {
    if features.is_empty() {
        return Ok(());
    }
    let base_query = format!(
        "INSERT INTO {}.{} (
            time, source, price, maker_quantity, taker_quantity, 
            obi, obi_005p, ob_spread,
            
            tib_id_hist, tib_imb_hist, tib_thres_hist, tib_aggr_hist, tib_aggr_v_hist, 
            tib_imb_curr, tib_thres_curr, tib_vwap_curr,            
            vmb_id_hist, vmb_imb_hist, vmb_thres_hist, vmb_aggr_hist, vmb_aggr_v_hist,
            vmb_imb_curr, vmb_thres_curr, vmb_vwap_curr, vmb_cvd_curr,
            
            dib_id_hist, dib_imb_hist, dib_thres_hist, dib_aggr_hist, dib_aggr_v_hist,
            dib_imb_curr, dib_thres_curr, dib_vwap_curr, dib_cvd_curr
        ) VALUES ",
        schema, table,
    );

    let mut placeholders: Vec<String> = Vec::new();
    let mut values: Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>> = Vec::new();

    let mut param_index = 1;
    for feature in features {
        placeholders.push(format!(
            "(
                to_timestamp(${}::FLOAT8), ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${},
                ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${},
                ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}
            )",
            param_index,     // time
            param_index + 1, // source
            param_index + 2, // price
            param_index + 3, // maker_quantity
            param_index + 4, // taker_quantity
            param_index + 5, // obi
            param_index + 6, // obi_005p
            param_index + 7, // ob_spread
            // Tick imbalance bar
            param_index + 8,  // id (tib_id_hist)
            param_index + 9,  // imb (tib_imb_hist)
            param_index + 10, // thres (tib_thres_hist)
            param_index + 11, // aggressiveness (tib_aggressiveness_hist)
            param_index + 12, // aggressiveness_volume (tib_aggressiveness_volume_hist)
            param_index + 13, // imb (tib_imb_curr)
            param_index + 14, // thres (tib_thres_curr)
            param_index + 15, // vwap (tib_vwap_curr)
            // Volume imbalance bar
            param_index + 16, // id (vmb_id_hist)
            param_index + 17, // imb (vmb_imb_hist)
            param_index + 18, // thres (vmb_thres_hist)
            param_index + 19, // aggressiveness (vmb_aggressiveness_hist)
            param_index + 20, // aggressiveness_volume (vmb_aggressiveness_volume_hist)
            param_index + 21, // imb (vmb_imb_curr)
            param_index + 22, // thres (vmb_thres_curr)
            param_index + 23, // vwap (vmb_vwap_curr)
            param_index + 24, // cvd (vmb_cvd_curr)
            // Dollar imbalance bar
            param_index + 25, // id (dib_id_hist)
            param_index + 26, // imb (dib_imb_hist)
            param_index + 27, // thres (dib_thres_hist)
            param_index + 28, // aggressiveness (dib_aggressiveness_hist)
            param_index + 29, // aggressiveness_volume (dib_aggressiveness_volume_hist)
            param_index + 30, // imb (dib_imb_curr)
            param_index + 31, // thres (dib_thres_curr)
            param_index + 32, // vwap (dib_vwap_curr)
            param_index + 33, // cvd (dib_cvd_curr)
        ));

        values.push(Box::new(feature.trade_time as f64 / 1000.0)); // time
        values.push(Box::new(feature.source.clone())); // source
        values.push(Box::new(feature.price)); // price
        values.push(Box::new(feature.maker_quantity)); // maker_quantity
        values.push(Box::new(feature.taker_quantity)); // taker_quantity
        values.push(Box::new(feature.obi)); // obi
        values.push(Box::new(feature.obi_range.0)); // obi_005p
        values.push(Box::new(feature.ob_spread)); // ob_spread

        // Tick imbalance bar
        values.push(Box::new(feature.tick_imbalance_bar.bar.id.clone())); // tib_id_hist
        values.push(Box::new(feature.tick_imbalance_bar.bar.imb)); // tib_imb_hist
        values.push(Box::new(feature.tick_imbalance_bar.bar.imb_thres)); // tib_thres_hist
        values.push(Box::new(feature.tick_imbalance_bar.bar.aggressive())); // tib_aggressiveness_hist
        values.push(Box::new(feature.tick_imbalance_bar.bar.aggressive_vol())); // tib_aggressiveness_volume_hist

        values.push(Box::new(feature.tick_imbalance)); // tib_imb_curr
        values.push(Box::new(feature.tick_imbalance_thres)); // tib_thres_curr
        values.push(Box::new(feature.tick_imbalance_vwap)); // tib_vwap_curr

        // Volume imbalance bar
        values.push(Box::new(feature.volume_imbalance_bar.bar.id.clone())); // vmb_id_hist
        values.push(Box::new(feature.volume_imbalance_bar.bar.imb)); // vmb_imb_hist
        values.push(Box::new(feature.volume_imbalance_bar.bar.imb_thres)); // vmb_thres_hist
        values.push(Box::new(feature.volume_imbalance_bar.bar.aggressive())); // vmb_aggressiveness_hist
        values.push(Box::new(feature.volume_imbalance_bar.bar.aggressive_vol())); // vmb_aggressiveness_volume_hist

        values.push(Box::new(feature.volume_imbalance)); // vmb_imb_curr
        values.push(Box::new(feature.volume_imbalance_thres)); // vmb_thres_curr
        values.push(Box::new(feature.volume_imbalance_vwap)); // vmb_vwap_curr
        values.push(Box::new(feature.volume_imbalance_cvd)); // vmb_cvd_curr

        // Dollar imbalance bar
        values.push(Box::new(feature.dollar_imbalance_bar.bar.id.clone())); // dib_id_hist
        values.push(Box::new(feature.dollar_imbalance_bar.bar.imb)); // dib_imb_hist
        values.push(Box::new(feature.dollar_imbalance_bar.bar.imb_thres)); // dib_thres_hist
        values.push(Box::new(feature.dollar_imbalance_bar.bar.aggressive())); // dib_aggressiveness_hist
        values.push(Box::new(feature.dollar_imbalance_bar.bar.aggressive_vol())); // dib_aggressiveness_volume_hist

        values.push(Box::new(feature.dollar_imbalance)); // dib_imb_curr
        values.push(Box::new(feature.dollar_imbalance_thres)); // dib_thres_curr
        values.push(Box::new(feature.dollar_imbalance_vwap)); // dib_vwap_curr
        values.push(Box::new(feature.dollar_imbalance_cvd)); // dib_cvd_curr

        param_index += 34;
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
