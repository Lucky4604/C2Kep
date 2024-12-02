

use std::error::Error;
use crate::data_synchronization::run_data_synchronization;

mod websocket_client;
mod opcua_client;
mod config;
mod queue;
mod producer;
mod consumer;
mod metrics;
mod data_synchronization; 

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Call the function that runs the entire synchronization process
    run_data_synchronization().await?;

    Ok(())
}
