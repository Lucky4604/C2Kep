use crate::queue::{Queue, QUEUE};
use crate::websocket_client::{OpcUaData, WebSocketClient, HistoricalValue, Historical, Universal};
use crate::metrics::Metrics;
use std::sync::Arc;
use prost::Message;
use tokio::sync::Mutex;
use tokio_native_tls::TlsStream;
use tokio::time::{sleep, Duration};

pub struct DataConsumer {
    queue: Arc<Queue<OpcUaData>>,
    ws_stream: Arc<Mutex<tokio_tungstenite::WebSocketStream<TlsStream<tokio::net::TcpStream>>>>,
    metrics: Arc<Metrics>,
}

impl DataConsumer {
    pub fn new(
        queue: Arc<Queue<OpcUaData>>,
        ws_stream: Arc<Mutex<tokio_tungstenite::WebSocketStream<TlsStream<tokio::net::TcpStream>>>>,
        metrics: Arc<Metrics>,
    ) -> Self {
        DataConsumer { queue, ws_stream, metrics }
    }


    pub async fn run(&self) {
        loop {
            if let Some(data) = self.consume_data().await {
                self.process_data(data).await;
            } else {
                self.wait_for_queue().await;
            }
        }
    }


    async fn consume_data(&self) -> Option<OpcUaData> {
        QUEUE.pop()
    }


    async fn process_data(&self, data: OpcUaData) {
        let historical_value = HistoricalValue {
            t: data.timestamp,
            v: data.value as f64,
        };

        let historical = Historical {
            batchid: 0,
            sensor: data.item,
            values: vec![historical_value],
        };

        let universal = Universal {
            type_: vec![7201; 1000],
            messages: vec![historical.encode_to_vec()],
        };

        if let Err(e) = self.send_data(universal).await {
            eprintln!("Error sending data to WebSocket: {}", e);
        } else {
            self.metrics.increment_consumed();
            println!("Consumed data: {}", self.metrics.get_consumed());
        }
    }


    async fn send_data(&self, universal: Universal) -> Result<(), Box<dyn std::error::Error>> {
        let mut ws_stream_guard = self.ws_stream.lock().await;
        WebSocketClient::send_protobuf_data(&mut ws_stream_guard, universal).await
    }


    async fn wait_for_queue(&self) {
        sleep(Duration::from_millis(10)).await;
    }
}



