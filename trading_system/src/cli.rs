// src/cli.rs
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "Trading Exchange")]
#[command(about = "A local persistent trading exchange", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Admin related commands
    Admin {
        #[command(subcommand)]
        command: AdminCommands,
    },
    /// Customer related commands
    Customer {
        #[command(subcommand)]
        command: CustomerCommands,
    },
    /// Regulator related commands
    Regulator {
        #[command(subcommand)]
        command: RegulatorCommands,
    },
}

#[derive(Subcommand)]
pub enum AdminCommands {
    CreateUser {
        username: String,
        password: String,
        role: String,
    },
    // Add more admin commands
}

#[derive(Subcommand)]
pub enum CustomerCommands {
    Buy {
        asset: String,
        amount: f64,
    },
    Sell {
        asset: String,
        amount: f64,
    },
    // Add more customer commands
}

#[derive(Subcommand)]
pub enum RegulatorCommands {
    ViewTransactions,
    ViewWallets,
    // Add more regulator commands
}