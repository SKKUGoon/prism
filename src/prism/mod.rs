pub mod bar;
pub mod bar_manager;
pub mod executor;
pub mod stream;

#[derive(Debug, Clone)]
pub enum AssetSource {
    Future,
    Spot,
}
