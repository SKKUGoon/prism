use crate::prism::bar_manager::Bar;
use crate::prism::engine::PrismaFeature;
use tokio::sync::mpsc;

#[derive(Debug)]
#[allow(dead_code)]
pub struct PrismConfig {
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

    pub fn enable_data_dump(&mut self) {
        self.data_dump = true;
    }
}

#[derive(Debug)]
#[allow(dead_code)]
struct TradeComputer {
    // TODO: Implement trade computer
    sentiment: f32, // -1 to 1
    risk: f32,      // 0 to 1
}

impl TradeComputer {
    pub fn new() -> Self {
        Self {
            sentiment: 0.0,
            risk: 0.0,
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct Prism {
    config: PrismConfig,

    // Read Features
    rx_fut_feature: mpsc::Receiver<PrismaFeature>, // Future
    rx_spt_feature: mpsc::Receiver<PrismaFeature>, // Spot
    fut_bar: Bar,
    spt_bar: Bar,

    // Trade Computer
    trade_computer: TradeComputer,

    // Write to database
    tx_fut_db: mpsc::Sender<PrismaFeature>,
    tx_spt_db: mpsc::Sender<PrismaFeature>,
}

impl Prism {
    pub fn new(
        config: PrismConfig,
        rx_fut_feature: mpsc::Receiver<PrismaFeature>,
        rx_spt_feature: mpsc::Receiver<PrismaFeature>,
        tx_fut_db: mpsc::Sender<PrismaFeature>,
        tx_spt_db: mpsc::Sender<PrismaFeature>,
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
                    self.fut_bar.update_tick_imbalance_bar(&feature.tick_imbalance_bar);
                    self.fut_bar.update_volume_imbalance_bar(&feature.volume_imbalance_bar_both);
                    self.fut_bar.update_volume_imbalance_bar(&feature.volume_imbalance_bar_maker);
                    self.fut_bar.update_volume_imbalance_bar(&feature.volume_imbalance_bar_taker);

                    if self.config.data_dump {
                        self.tx_fut_db.send(feature).await.unwrap();
                    }
                }
                Some(feature) = self.rx_spt_feature.recv() => {
                    self.spt_bar.update_tick_imbalance_bar(&feature.tick_imbalance_bar);
                    self.spt_bar.update_volume_imbalance_bar(&feature.volume_imbalance_bar_both);
                    self.spt_bar.update_volume_imbalance_bar(&feature.volume_imbalance_bar_maker);
                    self.spt_bar.update_volume_imbalance_bar(&feature.volume_imbalance_bar_taker);

                    if self.config.data_dump {
                        self.tx_spt_db.send(feature).await.unwrap();
                    }
                }
            }
        }
    }
}
