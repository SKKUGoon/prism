use crate::prism::bar_manager::Bar;
use crate::prism::engine::PrismaFeature;
use tokio::sync::mpsc;

#[derive(Debug)]
#[allow(dead_code)]
pub struct PrismTradeConfig {
    leverage: u8,
    max_leverage: u8,
    loss_cut: Option<f32>,
    take_profit: Option<f32>,
}

impl PrismTradeConfig {
    pub fn default() -> Self {
        Self {
            leverage: 1,
            max_leverage: 30,
            loss_cut: Some(0.5),
            take_profit: Some(0.10),
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct PrismTrade {
    config: PrismTradeConfig,

    // Read Features
    rx_fut_feature: mpsc::Receiver<PrismaFeature>, // Future
    rx_spt_feature: mpsc::Receiver<PrismaFeature>, // Spot
    fut_bar: Bar,
    spt_bar: Bar,
}

impl PrismTrade {
    pub fn new(
        config: PrismTradeConfig,
        rx_fut_feature: mpsc::Receiver<PrismaFeature>,
        rx_spt_feature: mpsc::Receiver<PrismaFeature>,
    ) -> Self {
        Self {
            config,
            rx_fut_feature,
            rx_spt_feature,

            fut_bar: Bar::new(100),
            spt_bar: Bar::new(100),
        }
    }

    pub async fn work(&mut self) {
        loop {
            tokio::select! {
                Some(feature) = self.rx_fut_feature.recv() => {
                    self.fut_bar.update_bar(&feature.tib);
                }
                Some(feature) = self.rx_spt_feature.recv() => {
                    self.spt_bar.update_bar(&feature.tib);
                }
            }
        }
    }
}
