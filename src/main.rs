use crate::data::{
    binance::BinanceStreams,
    exchanges::{FutureChannels, SpotChannels},
    upbit::UpbitStreams,
};
use channel::{FutureChannel, SpotChannel, SystemChannelPairs};
use config::read_env_config;
use database::postgres::timescale_batch_writer;
use log::{error, info, warn};
use prism::{
    executor::{PrismConfig, PrismTradeManager},
    stream::{future::FutureStream, spot::SpotStream, FutureReceivers},
};
use tokio::signal;

mod channel;
mod config;
mod data;
mod database;
mod prism;
mod trade;

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

    /* Create channels for thread communication */
    let binance_fut = FutureChannel::new(env_var.channel_capacity);
    let binance_spt = SpotChannel::new(env_var.channel_capacity);

    let upbit_spt_krw = SpotChannel::new(env_var.channel_capacity);
    let upbit_spt_btc = SpotChannel::new(env_var.channel_capacity);
    let upbit_spt_usdt = SpotChannel::new(env_var.channel_capacity);

    let system_fut = SystemChannelPairs::new(env_var.channel_capacity);
    let system_spt = SystemChannelPairs::new(env_var.channel_capacity);

    // let binance_streams = BinanceStreams::new(binance_fut, binance_spt);

    /* Start stream managers */
    let mut binance_future_manager = FutureStream::new(
        binance_fut.ob.mng.1,
        binance_fut.agg.1,
        FutureReceivers {
            rx_markprice: binance_fut.additional.mark.1,
            rx_liquidation: binance_fut.additional.liq.1,
        },
        system_fut.exec.0,
    );
    let mut binance_spot_manager = SpotStream::new(
        binance_spt.ob.mng.1,
        binance_spt.agg.1,
        system_spt.exec.0.clone(),
    );
    let mut upbit_spot_manager = SpotStream::new(
        upbit_spt_krw.ob.mng.1,
        upbit_spt_krw.agg.1,
        system_spt.exec.0.clone(),
    );

    /* Start Data Manager */
    let mut core_config = PrismConfig::default();
    core_config.enable_data_dump(env_var.data_dump);

    let mut core_mng = PrismTradeManager::new(
        core_config,
        system_fut.exec.1,
        system_spt.exec.1,
        system_fut.db.0,
        system_spt.db.0,
    );

    /* Feature Creation Engine Start */
    tasks.spawn(async move { binance_future_manager.work().await });
    tasks.spawn(async move { binance_spot_manager.work().await });
    tasks.spawn(async move { upbit_spot_manager.work().await });
    tasks.spawn(async move { core_mng.work().await });

    /* Timescale Insertion */
    if env_var.data_dump {
        tasks.spawn(async move {
            if let Err(e) =
                timescale_batch_writer("binance", &env_var.table_fut.clone(), system_fut.db.1).await
            {
                error!("Timescale batch writer error: {}", e);
            }
        });

        tasks.spawn(async move {
            if let Err(e) =
                timescale_batch_writer("binance", &env_var.table_spt.clone(), system_spt.db.1).await
            {
                error!("Timescale batch writer error: {}", e);
            }
        });
    }

    /* Start Data Streams */
    let binance_streams = BinanceStreams::new(
        FutureChannels {
            ob_raw_in: binance_fut.ob.raw.1,
            ob_raw_out: binance_fut.ob.raw.0,
            ob_mng_out: binance_fut.ob.mng.0,
            agg_out: binance_fut.agg.0,
            liq_out: binance_fut.additional.liq.0,
            mark_out: binance_fut.additional.mark.0,
        },
        SpotChannels {
            ob_raw_in: binance_spt.ob.raw.1,
            ob_raw_out: binance_spt.ob.raw.0,
            ob_mng_out: binance_spt.ob.mng.0,
            agg_out: binance_spt.agg.0,
        },
    );
    binance_streams.spawn_streams(
        &mut tasks,
        env_var.symbol_binance_fut.clone(),
        env_var.symbol_binance_spt.clone(),
    );

    let upbit_krw_streams = UpbitStreams::new(SpotChannels {
        ob_raw_in: upbit_spt_krw.ob.raw.1,
        ob_raw_out: upbit_spt_krw.ob.raw.0,
        ob_mng_out: upbit_spt_krw.ob.mng.0,
        agg_out: upbit_spt_krw.agg.0,
    });

    let upbit_btc_streams = UpbitStreams::new(SpotChannels {
        ob_raw_in: upbit_spt_btc.ob.raw.1,
        ob_raw_out: upbit_spt_btc.ob.raw.0,
        ob_mng_out: upbit_spt_btc.ob.mng.0,
        agg_out: upbit_spt_btc.agg.0,
    });

    let upbit_usdt_streams = UpbitStreams::new(SpotChannels {
        ob_raw_in: upbit_spt_usdt.ob.raw.1,
        ob_raw_out: upbit_spt_usdt.ob.raw.0,
        ob_mng_out: upbit_spt_usdt.ob.mng.0,
        agg_out: upbit_spt_usdt.agg.0,
    });

    upbit_krw_streams.spawn_streams(&mut tasks, env_var.symbol_upbit_krw.clone());
    upbit_btc_streams.spawn_streams(&mut tasks, env_var.symbol_upbit_btc.clone());
    upbit_usdt_streams.spawn_streams(&mut tasks, env_var.symbol_upbit_usdt.clone());

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
