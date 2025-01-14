// use log::{error, info};
// use tokio::sync::mpsc;
// use tokio_postgres::{Client, NoTls};

// use crate::prism::engine::PrismaFeature;

// #[allow(dead_code)]
// pub async fn connect_to_timescale() -> Result<Client, Box<dyn std::error::Error>> {
//     let connection_str =
//         "host=host.docker.internal port=10501 user=postgres password=postgres dbname=postgres";

//     let (client, connection) = tokio_postgres::connect(connection_str, NoTls).await?;

//     tokio::spawn(async move {
//         if let Err(e) = connection.await {
//             error!("Timescale connection error: {}", e);
//         }
//     });

//     Ok(client)
// }

// // pub async fn batch_insert_into_timescale(
// //     schema: &str,
// //     table: &str,
// //     client: &Client,
// //     features: &Vec<PrismaFeature>,
// // ) -> Result<(), Box<dyn std::error::Error>> {
// //     if features.is_empty() {
// //         return Ok(());
// //     }

// //     let base_query = format!(
// //         "INSERT INTO {}.{} (time, source, price, maker_quantity, taker_quantity, aggressive, ofi, obi, obi_001, obi_002, obi_005, obi_010) VALUES ",
// //         schema,
// //         table,
// //     );
// //     let mut placeholders: Vec<String> = Vec::new();
// //     let mut values: Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>> = Vec::new();

// //     let mut param_index = 1;
// //     for feature in features {
// //         placeholders.push(format!(
// //             "(to_timestamp(${}::FLOAT8), ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${})",
// //             param_index,
// //             param_index + 1,
// //             param_index + 2,
// //             param_index + 3,
// //             param_index + 4,
// //             param_index + 5,
// //             param_index + 6,
// //             param_index + 7,
// //             param_index + 8,
// //             param_index + 9,
// //             param_index + 10,
// //             param_index + 11
// //         ));
// //         values.push(Box::new(feature.time as f64 / 1000.0));
// //         values.push(Box::new(feature.source.clone()));
// //         values.push(Box::new(feature.price));
// //         values.push(Box::new(0.0)); // Empty
// //         values.push(Box::new(0.0)); // Empty
// //         values.push(Box::new(0.0)); // Empty
// //         values.push(Box::new(feature.ofi));
// //         values.push(Box::new(feature.obi));
// //         values.push(Box::new(feature.obi_range.0)); // Orderbook imbalance 0.01
// //         values.push(Box::new(feature.obi_range.1)); // Orderbook imbalance 0.02
// //         values.push(Box::new(feature.obi_range.2)); // Orderbook imbalance 0.05
// //         values.push(Box::new(feature.obi_range.3)); // Orderbook imbalance 0.10

// //         param_index += 12;
// //     }
// //     let combined_data = placeholders.join(",");
// //     let query = format!("{}{}", base_query, combined_data);
// //     client
// //         .execute(
// //             &query,
// //             &values
// //                 .iter()
// //                 .map(|p| p.as_ref() as &(dyn tokio_postgres::types::ToSql + Sync))
// //                 .collect::<Vec<_>>(),
// //         )
// //         .await?;

// //     Ok(())
// // }

// // pub async fn timescale_batch_writer(
// //     schema: &str,
// //     table: &str,
// //     mut rx: mpsc::Receiver<PrismaFeature>,
// // ) -> Result<(), Box<dyn std::error::Error>> {
// //     info!("Starting data insertion");

// //     let client = connect_to_timescale().await?;

// //     let mut buffer = Vec::new();

// //     while let Some(feature) = rx.recv().await {
// //         // Dynamic batch size adjustment based on the buffer usage
// //         let batch_size = if 2000 - rx.capacity() > 1000 { 100 } else { 50 };

// //         info!("Received from {:?}", feature.source.clone());

// //         buffer.push(feature);
// //         if buffer.len() >= batch_size {
// //             batch_insert_into_timescale(schema, table, &client, &buffer).await?;
// //             info!("Inserted {} features into timescale", buffer.len());
// //             buffer.clear();
// //         }
// //     }
// //     Ok(())
// // }
