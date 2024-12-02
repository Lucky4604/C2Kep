use crate::queue::Queue;
use crate::websocket_client::OpcUaData;
use crate::metrics::Metrics;
use tokio::time::{sleep, Duration};
use std::sync::Arc;




pub async fn run(queue: Arc<Queue<OpcUaData>>, metrics: Arc<Metrics>) {
    let _ = queue;
    let _ = metrics;
    loop {
       
        sleep(Duration::from_secs(1)).await;

    }
}


