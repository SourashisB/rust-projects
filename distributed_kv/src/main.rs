mod server;
mod client;

use clap::{Parser, Subcommand};
use std::error::Error;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Server {
        #[arg(short, long, default_value = "127.0.0.1:8080")]
        address: String,
    },
    Client {
        #[arg(short, long, default_value = "127.0.0.1:8080")]
        server: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let cli = Cli::parse();

    match &cli.command {
        Commands::Server { address } => {
            server::run_server(address).await?;
        }
        Commands::Client { server } => {
            client::run_client(server).await?;
        }
    }

    Ok(())
}