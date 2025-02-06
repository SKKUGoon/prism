pub mod bar_dollar_imbalance;
pub mod bar_manager;
pub mod bar_tick_imbalance;
pub mod bar_volume_imbalance;
pub mod executor;
pub mod stream_process;

#[derive(Debug, Clone)]
pub enum AssetSource {
    Future,
    Spot,
}

impl AssetSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            AssetSource::Future => "f",
            AssetSource::Spot => "s",
        }
    }
}
