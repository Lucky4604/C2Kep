use std::sync::Arc;
use tokio::task::spawn_blocking;
use tokio::sync::Mutex;
use crate::websocket_client::WebSocketClient;
use crate::opcua_client::start_kep_server;
use crate::queue:: QUEUE;
use crate::metrics::Metrics;
use crate::producer;
use crate::consumer::DataConsumer;

pub async fn run_data_synchronization() -> Result<(), Box<dyn std::error::Error>> {
    let (tags, ws_stream) = WebSocketClient::run().await?;

    let queue=Arc::clone(&QUEUE);
    let metrics = Arc::new(Metrics::new());

    let num_producer = 5;
    let num_consumer = 5;

    if !tags.is_empty() {
        println!("Starting OPC UA server with {} tags", tags.len());

        let queue_clone = Arc::clone(&queue);
        let metrics_clone = Arc::clone(&metrics);
        let ws_stream_clone = Arc::new(Mutex::new(ws_stream));

       
        for _ in 0..num_producer {
            tokio::spawn(producer::run(Arc::clone(&queue), Arc::clone(&metrics)));
        }

       
        for _ in 0..num_consumer {
            let data_consumer = Arc::new(DataConsumer::new(
                Arc::clone(&queue),
                Arc::clone(&ws_stream_clone),
                Arc::clone(&metrics),
            ));
            let data_consumer_clone = Arc::clone(&data_consumer);
            tokio::spawn(async move {
                data_consumer_clone.run().await;
            });
        }
      
        let ws_stream_for_kep = Arc::clone(&ws_stream_clone);
        spawn_blocking(move || {
            if let Err(e) = start_kep_server(tags, ws_stream_for_kep, queue_clone, metrics_clone) {
                eprintln!("Error starting OPC UA server: {}", e);
            }
        }).await?;

   
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                println!("Produced messages: {}", metrics.get_produced());
                println!("Consumed messages: {}", metrics.get_consumed());
            }
        });
    } else {
        println!("No valid tags found to subscribe to.");
    }

    Ok(())
}
