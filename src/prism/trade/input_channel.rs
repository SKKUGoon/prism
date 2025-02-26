use tokio::sync::mpsc;

/* Input Channels */
pub struct BinanceFeatureProcessed {
    // pub futures: mpsc::Receiver<FeatureProcessed>,
    // pub spot: mpsc::Receiver<FeatureProcessed>,
}

pub struct UpbitFeatureProcessed {
    // pub krw: mpsc::Receiver<FeatureProcessed>,
    // pub btc: mpsc::Receiver<FeatureProcessed>,
    // pub usdt: mpsc::Receiver<FeatureProcessed>,
}

/* Output Channels */
pub struct DatabaseRecord {
    // pub strat1: mpsc::Sender<SnipeLargeOrderParams>,
}
