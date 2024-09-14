mod cli;

use std::error::Error;

pub async fn run_client(server: &str) -> Result<(), Box<dyn Error>> {
    cli::start_cli(server).await
}