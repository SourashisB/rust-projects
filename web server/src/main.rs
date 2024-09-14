mod server;
mod handler;
mod router;
mod logger;

use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    logger::init()?;
    let server = server::Server::new("127.0.0.1:3000".to_string());
    server.run().await?;
    Ok(())
}