use std::error::Error;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

use super::storage::Storage;
use super::replication::Replication;

pub async fn start_server(
    address: &str,
    storage: Arc<Mutex<Storage>>,
    replication: Arc<Mutex<Replication>>,
) -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(address).await?;
    log::info!("Server listening on {}", address);

    loop {
        let (socket, _) = listener.accept().await?;
        let storage = Arc::clone(&storage);
        let replication = Arc::clone(&replication);

        tokio::spawn(async move {
            handle_connection(socket, storage, replication).await;
        });
    }
}

async fn handle_connection(
    mut socket: TcpStream,
    storage: Arc<Mutex<Storage>>,
    replication: Arc<Mutex<Replication>>,
) {
    let (reader, mut writer) = socket.split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    while let Ok(bytes_read) = reader.read_line(&mut line).await {
        if bytes_read == 0 {
            break;
        }

        let response = process_command(&line, &storage, &replication).await;
        writer.write_all(response.as_bytes()).await.unwrap();
        writer.flush().await.unwrap();

        line.clear();
    }
}

async fn process_command(
    command: &str,
    storage: &Arc<Mutex<Storage>>,
    replication: &Arc<Mutex<Replication>>,
) -> String {
    let parts: Vec<&str> = command.trim().split_whitespace().collect();

    match parts.as_slice() {
        ["GET", key] => {
            let storage = storage.lock().await;
            match storage.get(key) {
                Some(value) => format!("Value: {}\n", value),
                None => "Key not found\n".to_string(),
            }
        }
        ["SET", key, value] => {
            let mut storage = storage.lock().await;
            storage.set(key, value);
            replication.lock().await.replicate(&storage);
            "OK\n".to_string()
        }
        ["DELETE", key] => {
            let mut storage = storage.lock().await;
            storage.delete(key);
            replication.lock().await.replicate(&storage);
            "OK\n".to_string()
        }
        ["LIST"] => {
            let storage = storage.lock().await;
            let all_pairs = storage.list_all();
            if all_pairs.is_empty() {
                "No key-value pairs stored\n".to_string()
            } else {
                let mut response = String::from("Stored key-value pairs:\n");
                for (key, value) in all_pairs {
                    response.push_str(&format!("{}: {}\n", key, value));
                }
                response
            }
        }
        _ => "Invalid command\n".to_string(),
    }
}