use crate::prism::stream::FeatureProcessed;
use tokio::sync::mpsc;

pub struct BinanceFeatureProcessed {
    pub futures: mpsc::Receiver<FeatureProcessed>,
    pub spot: mpsc::Receiver<FeatureProcessed>,
}
