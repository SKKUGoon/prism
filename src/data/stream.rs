use std::future::Future;
use tokio_tungstenite::tungstenite;

pub trait StreamHandler {
    fn connect(&self) -> Box<dyn Future<Output = Result<(), tungstenite::Error>> + Send + Unpin>;
}
