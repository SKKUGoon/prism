use std::future::Future;
use tokio_tungstenite::tungstenite;

#[allow(dead_code)]
pub trait StreamHandler {
    fn connect(&self) -> Box<dyn Future<Output = Result<(), tungstenite::Error>> + Send + Unpin>;
}
