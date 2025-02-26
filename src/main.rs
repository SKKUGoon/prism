use crate::data::{
    binance::BinanceThreads,
    exchanges::{FutureDataChannels, SpotDataChannels},
    upbit::UpbitThreads,
};
use channel::{Future, FutureChannel, SpotChannel};
use config::read_env_config;
use database::postgres::connect_to_timescale;
use log::{error, info, warn};
use prism::core::{future::FutureCore, spot::SpotCore, Core};
use tokio::signal;

mod channel;
mod config;
mod data;
mod database;
mod prism;

/*
(1) Data flows like:
     Websocket -> Data -> Engine -> Executor (or database) -> Trade order
(2) Data from websocket to engine.
     The channel name will be: tx(rx)_(fut/spt)_(ob/agg)_data
     a. (ob)Orderbook
     b. (agg)Aggtrade - AggTrade does not need to be processed by class. So `rx` part goes to straight to engine
(3) Data engineering inside engine. Send to Executor (or database)
     The channel name will be: tx(rx)_(fut/spt)_exec
*/

#[tokio::main]
async fn main() {
    /* Initialize logger */
    env_logger::init();
    let env_var = read_env_config();

    /* Create threaded task set */
    let mut tasks = tokio::task::JoinSet::new();

    /* Create channels and data managers for thread communication */
    let binance_fut = FutureChannel::new(env_var.channel_capacity);
    let mut binance_future_core = Core::<FutureCore>::new(
        binance_fut.ob.1,
        binance_fut.agg.1,
        binance_fut.additional.mark.1,
        binance_fut.additional.liq.1,
    );

    let binance_spt = SpotChannel::new(env_var.channel_capacity);
    let mut binance_spot_core = Core::<SpotCore>::new(binance_spt.ob.1, binance_spt.agg.1);

    let upbit_spt_krw = SpotChannel::new(env_var.channel_capacity);
    let mut upbit_spot_core = Core::<SpotCore>::new(upbit_spt_krw.ob.1, upbit_spt_krw.agg.1);

    /* Feature Creation Engine */
    tasks.spawn(async move { binance_future_core.work().await });
    tasks.spawn(async move { binance_spot_core.work().await });
    tasks.spawn(async move { upbit_spot_core.work().await });

    // /* Start Data Manager */
    // let mut core_config = TradeConfig::default();
    // core_config.enable_data_dump(env_var.data_dump);
    // let mut trade_mng = TradeManager::new(
    //     BinanceFeatureProcessed {
    //         futures: binance_sys_fut.exec.1,
    //         spot: binance_sys_spt.exec.1,
    //     },
    //     UpbitFeatureProcessed {
    //         krw: upbit_krw_sys_spt.exec.1,
    //         btc: upbit_btc_sys_spt.exec.1, // If needed add `btc` and `usdt` streams
    //         usdt: upbit_usdt_sys_spt.exec.1,
    //     },
    //     DatabaseRecord {
    //         strat1: strat1.db.0,
    //     },
    //     core_config,
    // );
    // tasks.spawn(async move { trade_mng.work().await });

    // /* Timescale Insertion */
    // if env_var.data_dump {
    //     let client = connect_to_timescale().await.unwrap();
    //     tasks.spawn(async move {
    //         if let Err(e) =
    //             record_snipe_large_order_params(&client, &env_var.table_strat1, strat1.db.1).await
    //         {
    //             error!("Timescale batch writer error: {}", e);
    //         }
    //     });
    // }

    /* Start Data Streams */
    let binance_streams = BinanceThreads::new(
        FutureDataChannels {
            ob_out: binance_fut.ob.0,
            agg_out: binance_fut.agg.0,
            liq_out: binance_fut.additional.liq.0,
            mark_out: binance_fut.additional.mark.0,
        },
        SpotDataChannels {
            ob_out: binance_spt.ob.0,
            agg_out: binance_spt.agg.0,
        },
    );
    binance_streams.spawn_streams(
        &mut tasks,
        env_var.symbol_binance_fut.clone(),
        env_var.symbol_binance_spt.clone(),
    );

    let upbit_krw_streams = UpbitThreads::new(SpotDataChannels {
        ob_out: upbit_spt_krw.ob.0,
        agg_out: upbit_spt_krw.agg.0,
    });
    upbit_krw_streams.spawn_streams(&mut tasks, env_var.symbol_upbit_krw.clone());

    /* Graceful Shutdown */
    tokio::select! {
        _ = signal::ctrl_c() => {
            info!("Received Ctrl+C, shutting down...");
        }
        Some(res) = tasks.join_next() => {
            match res {
                Ok(_) => warn!("A task completed unexpectedly"),
                Err(e) => error!("A task failed: {}", e),
            }
        }
    }

    // Abort all remaining tasks
    tasks.abort_all();
    while tasks.join_next().await.is_some() {
        // Wait for all tasks to complete or be aborted
    }
    info!("Shutdown complete");
}
