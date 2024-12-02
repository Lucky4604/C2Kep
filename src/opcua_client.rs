use opcua_client::prelude::*;
use std::sync::{Arc, RwLock};
use tokio::sync::Mutex as TokioMutex;
use crate::websocket_client::OpcUaData;
use chrono::{DateTime, Utc};
use crate::config;
use crate::queue::{Queue, QUEUE};
use crate::metrics::Metrics;

pub fn start_kep_server(
    tags: Vec<String>,
    ws_stream: Arc<TokioMutex<tokio_tungstenite::WebSocketStream<tokio_native_tls::TlsStream<tokio::net::TcpStream>>>>,
    queue: Arc<Queue<OpcUaData>>,
    metrics: Arc<Metrics>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut client = ClientBuilder::new()
        .application_name("My First Client")
        .application_uri("urn:MyFirstClient")
        .create_sample_keypair(true)
        .trust_server_certs(false)
        .session_retry_limit(3)
        .client()
        .unwrap();

    let config = config::get_config();
    let endpoint: EndpointDescription = (
        config.opcua_url,
        "None",
        MessageSecurityMode::None,
        UserTokenPolicy::anonymous(),
    ).into();

    println!("Connecting to endpoint: {}", endpoint.endpoint_url);

    let session = client.connect_to_endpoint(endpoint, IdentityToken::Anonymous)?;

    let mut count = 0;
    const BATCH_SIZE: usize = 1000; 

   
    let tag_batches = tags.chunks(BATCH_SIZE);
    for batch in tag_batches {
        let ws_stream_clone = Arc::clone(&ws_stream);
        let queue_clone = Arc::clone(&queue);
        let metrics_clone = Arc::clone(&metrics);
        for tag in batch {
            let full_tag = format!("Channel1000.Device1000.{}", tag);
            if subscribe_to_values(session.clone(), full_tag.clone(), ws_stream_clone.clone(), queue_clone.clone(), metrics_clone.clone()).is_ok() {
                count += 1;
                println!("Successfully subscribed to tag: {}. Total successful subscriptions: {}", full_tag, count);
            } else {
                println!("Error creating subscription for tag: {}", full_tag);
            }
        }
    }

    println!("Total successful subscriptions: {}", count);

    Session::run(session);

    Ok(())
}

fn subscribe_to_values(
    session: Arc<RwLock<Session>>,
    tag: String,
    ws_stream: Arc<TokioMutex<tokio_tungstenite::WebSocketStream<tokio_native_tls::TlsStream<tokio::net::TcpStream>>>>,
    queue: Arc<Queue<OpcUaData>>,
    metrics: Arc<Metrics>,
) -> Result<(), StatusCode> {
    let _ = ws_stream; 

    let session = session.write().unwrap();

    let subscription_id = session.create_subscription(2000.0, 10, 30, 0, 0, true, DataChangeCallback::new(move |changed_monitored_items| {
        
        // println!("Data change from server:");
        for item in changed_monitored_items.iter() {
            if let Some(data) = process_value(item) {
                if let Err(e) = QUEUE.push(data) {
                
                    eprintln!("Error pushing data to queue: {}", e);
                } else {
                    metrics.increment_produced();
                    println!("Produced data: {}", metrics.get_produced());
                    
                }
            }
        }
    }))?;

    let node_id = NodeId::new(2, tag); 
    let items_to_create = vec![node_id.into()];

 
    session.create_monitored_items(subscription_id, TimestampsToReturn::Both, &items_to_create)?;

    Ok(())
}

fn datetime_to_timestamp_millis(datetime_str: &str) -> Option<i64> {
    if let Ok(datetime) = DateTime::parse_from_rfc3339(datetime_str) {
        let timestamp_millis = datetime.with_timezone(&Utc).timestamp_millis();
        Some(timestamp_millis)
    } else {
        None
    }
}

fn process_value(item: &MonitoredItem) -> Option<OpcUaData> {
    let node_id = &item.item_to_monitor().node_id;
    let data_value = item.last_value();
    match (data_value.value.clone(), data_value.server_timestamp) {
        (Some(value), Some(timestamp)) => {         
            if let Some(timestamp_millis) = datetime_to_timestamp_millis(&timestamp.to_string()) {
                if let Some(float_value) = value.as_f64() {
                    let opc_ua_data = OpcUaData {
                        item: node_id.to_string(),
                        value: float_value as f32, 
                        timestamp: timestamp_millis,
                    };
                    // println!(
                    //     "Item: \"{}\"\n Value: {:?}\n Timestamp in Millis: {}",
                    //     node_id,
                    //     float_value,
                    //     timestamp_millis
                    // );

                    Some(opc_ua_data) 
                } else {
                    println!("Value is not a float: {:?}", value);
                    None
                }
            } else {
                println!("Invalid timestamp format");
                None
            }
        },
        _ => {
            println!("Invalid data or timestamp");
            None
        }
    }
}
