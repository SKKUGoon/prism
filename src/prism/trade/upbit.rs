use crate::prism::stream::FeatureProcessed;
use tokio::sync::mpsc;

pub struct UpbitFeatureProcessed {
    pub krw: mpsc::Receiver<FeatureProcessed>,
    pub btc: mpsc::Receiver<FeatureProcessed>,
    pub usdt: mpsc::Receiver<FeatureProcessed>,
}
