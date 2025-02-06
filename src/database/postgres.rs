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
            
            tib_id_hist, tib_imb_hist, tib_thres_hist, tib_aggressiveness_hist, tib_aggressiveness_volume_hist, 
            tib_imb_curr, tib_thres_curr, tib_vwap_curr,
            
            vmb_id_hist, vmb_imb_hist, vmb_thres_hist, vmb_aggressiveness_hist, vmb_aggressiveness_volume_hist,
            vmb_imb_curr, vmb_thres_curr, vmb_vwap_curr,
            
            vmm_id_hist, vmm_imb_hist, vmm_thres_hist, vmm_aggressiveness_hist, vmm_aggressiveness_volume_hist,
            vmm_imb_curr, vmm_thres_curr, vmm_vwap_curr,
            vmm_aggressiveness_hist, vmm_aggressiveness_volume_hist,
            
            vmt_id_hist, vmt_imb_hist, vmt_thres_hist, vmt_aggressiveness_hist, vmt_aggressiveness_volume_hist,
            vmt_imb_curr, vmt_thres_curr, vmt_vwap_curr,
            
            dib_id_hist, dib_imb_hist, dib_thres_hist, dib_aggressiveness_hist, dib_aggressiveness_volume_hist,
            dib_imb_curr, dib_thres_curr, dib_vwap_curr
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
                ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${},
                ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}
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
            // Volume imbalance bar maker
            param_index + 24, // id (vmm_id_hist)
            param_index + 25, // imb (vmm_imb_hist)
            param_index + 26, // thres (vmm_thres_hist)
            param_index + 27, // aggressiveness (vmm_aggressiveness_hist)
            param_index + 28, // aggressiveness_volume (vmm_aggressiveness_volume_hist)
            param_index + 29, // imb (vmm_imb_curr)
            param_index + 30, // thres (vmm_thres_curr)
            param_index + 31, // vwap (vmm_vwap_curr)
            // Volume imbalance bar taker
            param_index + 32, // id (vmt_id_hist)
            param_index + 33, // imb (vmt_imb_hist)
            param_index + 34, // thres (vmt_thres_hist)
            param_index + 35, // aggressiveness (vmt_aggressiveness_hist)
            param_index + 36, // aggressiveness_volume (vmt_aggressiveness_volume_hist)
            param_index + 37, // imb (vmt_imb_curr)
            param_index + 38, // thres (vmt_thres_curr)
            param_index + 39, // vwap (vmt_vwap_curr)
            // Dollar imbalance bar
            param_index + 40, // id (dib_id_hist)
            param_index + 41, // imb (dib_imb_hist)
            param_index + 42, // thres (dib_thres_hist)
            param_index + 43, // aggressiveness (dib_aggressiveness_hist)
            param_index + 44, // aggressiveness_volume (dib_aggressiveness_volume_hist)
            param_index + 45, // imb (dib_imb_curr)
            param_index + 46, // thres (dib_thres_curr)
            param_index + 47, // vwap (dib_vwap_curr)
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
        values.push(Box::new(feature.tick_imbalance_bar.id.clone())); // tib_id_hist
        values.push(Box::new(feature.tick_imbalance_bar.imb)); // tib_imb_hist
        values.push(Box::new(feature.tick_imbalance_bar.imb_thres)); // tib_thres_hist
        values.push(Box::new(feature.tick_imbalance_bar.aggressive())); // tib_aggressiveness_hist
        values.push(Box::new(feature.tick_imbalance_bar.aggressive_vol())); // tib_aggressiveness_volume_hist

        values.push(Box::new(feature.tick_imbalance)); // tib_imb_curr
        values.push(Box::new(feature.tick_imbalance_thres)); // tib_thres_curr
        values.push(Box::new(feature.tick_imbalance_vwap)); // tib_vwap_curr

        // Volume imbalance bar
        values.push(Box::new(feature.volume_imbalance_bar_both.id.clone())); // vmb_id_hist
        values.push(Box::new(feature.volume_imbalance_bar_both.imb)); // vmb_imb_hist
        values.push(Box::new(feature.volume_imbalance_bar_both.imb_thres)); // vmb_thres_hist
        values.push(Box::new(feature.volume_imbalance_bar_both.aggressive())); // vmb_aggressiveness_hist
        values.push(Box::new(feature.volume_imbalance_bar_both.aggressive_vol())); // vmb_aggressiveness_volume_hist

        values.push(Box::new(feature.volume_imbalance_both)); // vmb_imb_curr
        values.push(Box::new(feature.volume_imbalance_both_thres)); // vmb_thres_curr
        values.push(Box::new(feature.volume_imbalance_both_vwap)); // vmb_vwap_curr

        // Volume imbalance bar maker
        values.push(Box::new(feature.volume_imbalance_bar_maker.id.clone())); // vmm_id_hist
        values.push(Box::new(feature.volume_imbalance_bar_maker.imb)); // vmm_imb_hist
        values.push(Box::new(feature.volume_imbalance_bar_maker.imb_thres)); // vmm_thres_hist
        values.push(Box::new(feature.volume_imbalance_bar_maker.aggressive())); // vmm_aggressiveness_hist
        values.push(Box::new(
            feature.volume_imbalance_bar_maker.aggressive_vol(),
        )); // vmm_aggressiveness_volume_hist

        values.push(Box::new(feature.volume_imbalance_maker)); // vmm_imb_curr
        values.push(Box::new(feature.volume_imbalance_maker_thres)); // vmm_thres_curr
        values.push(Box::new(feature.volume_imbalance_maker_vwap)); // vmm_vwap_curr

        // Volume imbalance bar taker
        values.push(Box::new(feature.volume_imbalance_bar_taker.id.clone())); // vmt_id_hist
        values.push(Box::new(feature.volume_imbalance_bar_taker.imb)); // vmt_imb_hist
        values.push(Box::new(feature.volume_imbalance_bar_taker.imb_thres)); // vmt_thres_hist
        values.push(Box::new(feature.volume_imbalance_bar_taker.aggressive())); // vmt_aggressiveness_hist
        values.push(Box::new(
            feature.volume_imbalance_bar_taker.aggressive_vol(),
        )); // vmt_aggressiveness_volume_hist

        values.push(Box::new(feature.volume_imbalance_taker)); // vmt_imb_curr
        values.push(Box::new(feature.volume_imbalance_taker_thres)); // vmt_thres_curr
        values.push(Box::new(feature.volume_imbalance_taker_vwap)); // vmt_vwap_curr

        // Dollar imbalance bar
        values.push(Box::new(feature.dollar_imbalance_bar_both.id.clone())); // dib_id_hist
        values.push(Box::new(feature.dollar_imbalance_bar_both.imb)); // dib_imb_hist
        values.push(Box::new(feature.dollar_imbalance_bar_both.imb_thres)); // dib_thres_hist
        values.push(Box::new(feature.dollar_imbalance_bar_both.aggressive())); // dib_aggressiveness_hist
        values.push(Box::new(feature.dollar_imbalance_bar_both.aggressive_vol())); // dib_aggressiveness_volume_hist

        values.push(Box::new(feature.dollar_imbalance)); // dib_imb_curr
        values.push(Box::new(feature.dollar_imbalance_thres)); // dib_thres_curr
        values.push(Box::new(feature.dollar_imbalance_both_vwap)); // dib_vwap_curr

        param_index += 48;
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
