use std::collections::VecDeque;

use crate::prism::bar_manager::Bar;
use crate::prism::stream_process::FeatureProcessed;
use crate::prism::AssetSource;
use tokio::sync::mpsc;

#[derive(Debug)]
#[allow(dead_code)]
pub struct PrismConfig {
    // Constants
    leverage: u8,
    max_leverage: u8,
    loss_cut: Option<f32>,
    take_profit: Option<f32>,
    trade_fees: f32,
    data_dump: bool,
}

impl PrismConfig {
    pub fn default() -> Self {
        Self {
            leverage: 1,
            max_leverage: 30,
            loss_cut: Some(0.05),
            take_profit: Some(0.10),
            trade_fees: 0.0004, // 0.04% for market takers
            data_dump: false,
        }
    }

    #[allow(dead_code)]
    pub fn enable_data_dump(&mut self) {
        self.data_dump = true;
    }
}

#[derive(Debug)]
#[allow(dead_code)]
struct TradeComputer {
    // Price information
    future_price: Option<f32>,
    spot_price: Option<f32>,
    future_spot_divergence: Option<f32>, // Divergence between future and spot

    /* VWAP */
    // 1. Time bar
    future_vwap: Option<f32>,
    spot_vwap: Option<f32>,
    historical_future_vwap: VecDeque<f32>,
    historical_spot_vwap: VecDeque<f32>,

    // 2. Tick Bar
    future_tick_vwap: Option<f32>,
    spot_tick_vwap: Option<f32>,
    historical_future_tick_vwap: VecDeque<f32>,
    historical_spot_tick_vwap: VecDeque<f32>,

    // 3. Volume Bar
    future_volume_vwap: Option<f32>,
    spot_volume_vwap: Option<f32>,
    historical_future_volume_vwap: VecDeque<f32>,
    historical_spot_volume_vwap: VecDeque<f32>,

    // 4. Dollar Bar
    future_dollar_vwap: Option<f32>,
    spot_dollar_vwap: Option<f32>,
    historical_future_dollar_vwap: VecDeque<f32>,
    historical_spot_dollar_vwap: VecDeque<f32>,

    // Information bars
    fut_bar: Bar, // Historical bars
    spt_bar: Bar,

    // Imbalances
    future_tick_imbalance: Option<f32>,
    future_tick_imbalance_thres: Option<f32>,
    spot_tick_imbalance: Option<f32>,
    spot_tick_imbalance_thres: Option<f32>,
    historical_future_tick_imbalance: Vec<f32>, // Within the bar
    historical_spot_tick_imbalance: Vec<f32>,   // Within the bar

    future_volume_imbalance: Option<f32>,
    future_volume_imbalance_thres: Option<f32>,
    spot_volume_imbalance: Option<f32>,
    spot_volume_imbalance_thres: Option<f32>,
    historical_future_volume_imbalance: Vec<f32>, // Within the bar
    historical_spot_volume_imbalance: Vec<f32>,   // Within the bar

    future_dollar_imbalance: Option<f32>,
    future_dollar_imbalance_thres: Option<f32>,
    spot_dollar_imbalance: Option<f32>,
    spot_dollar_imbalance_thres: Option<f32>,
    historical_future_dollar_imbalance: Vec<f32>, // Within the bar
    historical_spot_dollar_imbalance: Vec<f32>,   // Within the bar
}

impl TradeComputer {
    pub fn new() -> Self {
        Self {
            // Price information
            future_price: None,
            spot_price: None,
            future_spot_divergence: None,

            // VWAP
            future_vwap: None,
            spot_vwap: None,
            historical_future_vwap: VecDeque::new(),
            historical_spot_vwap: VecDeque::new(),

            // Tick VWAP
            future_tick_vwap: None,
            spot_tick_vwap: None,
            historical_future_tick_vwap: VecDeque::new(),
            historical_spot_tick_vwap: VecDeque::new(),

            // Volume VWAP
            future_volume_vwap: None,
            spot_volume_vwap: None,
            historical_future_volume_vwap: VecDeque::new(),
            historical_spot_volume_vwap: VecDeque::new(),

            // Dollar VWAP
            future_dollar_vwap: None,
            spot_dollar_vwap: None,
            historical_future_dollar_vwap: VecDeque::new(),
            historical_spot_dollar_vwap: VecDeque::new(),

            // Information bars
            fut_bar: Bar::new(100),
            spt_bar: Bar::new(100),

            // Imbalances
            future_tick_imbalance: None,
            future_tick_imbalance_thres: None,
            spot_tick_imbalance: None,
            spot_tick_imbalance_thres: None,
            historical_future_tick_imbalance: Vec::new(),
            historical_spot_tick_imbalance: Vec::new(),

            future_volume_imbalance: None,
            future_volume_imbalance_thres: None,
            spot_volume_imbalance: None,
            spot_volume_imbalance_thres: None,
            historical_future_volume_imbalance: Vec::new(),
            historical_spot_volume_imbalance: Vec::new(),

            future_dollar_imbalance: None,
            future_dollar_imbalance_thres: None,
            spot_dollar_imbalance: None,
            spot_dollar_imbalance_thres: None,
            historical_future_dollar_imbalance: Vec::new(),
            historical_spot_dollar_imbalance: Vec::new(),
        }
    }

    pub fn update_future_bars(&mut self, data: &FeatureProcessed) {
        self.fut_bar
            .update_tick_imbalance_bar(&data.tick_imbalance_bar);
        self.fut_bar
            .update_volume_imbalance_bar(&data.volume_imbalance_bar_both);
        self.fut_bar
            .update_volume_imbalance_bar(&data.volume_imbalance_bar_maker);
        self.fut_bar
            .update_volume_imbalance_bar(&data.volume_imbalance_bar_taker);
        self.fut_bar
            .update_dollar_imbalance_bar(&data.dollar_imbalance_bar_both);
    }

    pub fn update_spot_bars(&mut self, data: &FeatureProcessed) {
        self.spt_bar
            .update_tick_imbalance_bar(&data.tick_imbalance_bar);
        self.spt_bar
            .update_volume_imbalance_bar(&data.volume_imbalance_bar_both);
        self.spt_bar
            .update_volume_imbalance_bar(&data.volume_imbalance_bar_maker);
        self.spt_bar
            .update_volume_imbalance_bar(&data.volume_imbalance_bar_taker);
        self.spt_bar
            .update_dollar_imbalance_bar(&data.dollar_imbalance_bar_both);
    }

    pub fn update_trade_params(&mut self, data: &FeatureProcessed, source: AssetSource) {
        match source {
            AssetSource::Future => {
                self.future_price = Some(data.price);
                self.future_tick_vwap = Some(data.tick_imbalance_vwap);
                self.future_volume_vwap = Some(data.volume_imbalance_vwap_both);
                self.future_dollar_vwap = Some(data.dollar_imbalance_vwap_both);

                self.future_tick_imbalance = Some(data.tick_imbalance);
                self.future_volume_imbalance = Some(data.volume_imbalance_both);
                self.future_dollar_imbalance = Some(data.dollar_imbalance);

                self.historical_future_tick_vwap
                    .push_back(data.tick_imbalance_vwap);
                self.historical_future_volume_vwap
                    .push_back(data.volume_imbalance_vwap_both);
                self.historical_future_dollar_vwap
                    .push_back(data.dollar_imbalance_vwap_both);

                self.future_tick_imbalance_thres = Some(data.tick_imbalance_thres);
                self.future_volume_imbalance_thres = Some(data.volume_imbalance_both_thres);
                self.future_dollar_imbalance_thres = Some(data.dollar_imbalance_thres);

                // self.historical_future_tick_imbalance
                //     .push(data.tick_imbalance);
                // self.historical_future_volume_imbalance
                //     .push(data.volume_imbalance_both);
                // self.historical_future_dollar_imbalance
                //     .push(data.dollar_imbalance);
            }
            AssetSource::Spot => {
                self.spot_price = Some(data.price);
                self.spot_tick_vwap = Some(data.tick_imbalance_vwap);
                self.spot_volume_vwap = Some(data.volume_imbalance_vwap_both);
                self.spot_dollar_vwap = Some(data.dollar_imbalance_vwap_both);

                self.spot_tick_imbalance = Some(data.tick_imbalance);
                self.spot_volume_imbalance = Some(data.volume_imbalance_both);
                self.spot_dollar_imbalance = Some(data.dollar_imbalance);

                self.historical_spot_tick_vwap
                    .push_back(data.tick_imbalance_vwap);
                self.historical_spot_volume_vwap
                    .push_back(data.volume_imbalance_vwap_both);
                self.historical_spot_dollar_vwap
                    .push_back(data.dollar_imbalance_vwap_both);

                self.spot_tick_imbalance_thres = Some(data.tick_imbalance_thres);
                self.spot_volume_imbalance_thres = Some(data.volume_imbalance_both_thres);
                self.spot_dollar_imbalance_thres = Some(data.dollar_imbalance_thres);

                // self.historical_spot_tick_imbalance
                //     .push(data.tick_imbalance);
                // self.historical_spot_volume_imbalance
                //     .push(data.volume_imbalance_both);
                // self.historical_spot_dollar_imbalance
                //     .push(data.dollar_imbalance);
            }
        }

        if let (Some(f), Some(s)) = (self.future_price, self.spot_price) {
            self.future_spot_divergence = Some(f - s);
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct PrismTradeManager {
    config: PrismConfig,

    // Read Features
    rx_fut_feature: mpsc::Receiver<FeatureProcessed>, // Future
    rx_spt_feature: mpsc::Receiver<FeatureProcessed>, // Spot
    fut_bar: Bar,
    spt_bar: Bar,

    // Trade Computer
    trade_computer: TradeComputer,

    // Write to database
    tx_fut_db: mpsc::Sender<FeatureProcessed>,
    tx_spt_db: mpsc::Sender<FeatureProcessed>,
}

impl PrismTradeManager {
    pub fn new(
        config: PrismConfig,
        rx_fut_feature: mpsc::Receiver<FeatureProcessed>,
        rx_spt_feature: mpsc::Receiver<FeatureProcessed>,
        tx_fut_db: mpsc::Sender<FeatureProcessed>,
        tx_spt_db: mpsc::Sender<FeatureProcessed>,
    ) -> Self {
        Self {
            config,
            rx_fut_feature,
            rx_spt_feature,

            fut_bar: Bar::new(100),
            spt_bar: Bar::new(100),
            trade_computer: TradeComputer::new(),
            tx_fut_db,
            tx_spt_db,
        }
    }

    pub async fn work(&mut self) {
        loop {
            tokio::select! {
                Some(feature) = self.rx_fut_feature.recv() => {
                    self.trade_computer.update_future_bars(&feature);
                    self.trade_computer.update_trade_params(&feature, AssetSource::Future);

                    println!(
                        "from trade to event : {:?}ms | from event to processed : {:?}ms",
                        feature.event_time.saturating_sub(feature.trade_time),
                        feature.processed_time.saturating_sub(feature.event_time),
                    );
                    println!("--------------------------------");

                    if self.config.data_dump {
                        self.tx_fut_db.send(feature).await.unwrap();
                    }
                }

                Some(feature) = self.rx_spt_feature.recv() => {
                    self.trade_computer.update_spot_bars(&feature);
                    self.trade_computer.update_trade_params(&feature, AssetSource::Spot);

                    if self.config.data_dump {
                        self.tx_spt_db.send(feature).await.unwrap();
                    }
                }
            }
        }
    }
}
