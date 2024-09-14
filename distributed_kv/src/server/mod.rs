mod network;
mod storage;
mod replication;

use std::error::Error;
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn run_server(address: &str) -> Result<(), Box<dyn Error>> {
    let storage = Arc::new(Mutex::new(storage::Storage::new()));
    let replication = Arc::new(Mutex::new(replication::Replication::new()));

    network::start_server(address, storage, replication).await?;

    Ok(())
}