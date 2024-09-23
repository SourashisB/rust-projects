// src/main.rs
mod cli;
mod db;
mod handlers;
mod models;
mod utils;

use crate::cli::{Cli, Commands};
use crate::db::DB;
use crate::handlers::admin;
use crate::handlers::customer;
use crate::handlers::regulator;
use crate::utils::authenticate;
use clap::Parser;
use log::{error, info};
use std::process;

fn main() {
    // Initialize logging
    env_logger::init();

    // Parse CLI arguments
    let cli = Cli::parse();

    // Initialize database
    let db = match DB::new("data/exchange.db") {
        Ok(db) => db,
        Err(e) => {
            error!("Failed to initialize database: {}", e);
            process::exit(1);
        }
    };

    // For simplicity, assume user logs in via environment variables or config
    // Here we'll simulate authentication
    let username = "admin";
    let password = "adminpass";

    let user = match authenticate(&db, username, password) {
        Ok(Some((id, role))) => (id, role),
        Ok(None) => {
            error!("Authentication failed");
            process::exit(1);
        }
        Err(e) => {
            error!("Error during authentication: {}", e);
            process::exit(1);
        }
    };

    // Handle commands based on user role
    match &cli.command {
        Commands::Admin { command } if user.1.to_lowercase() == "admin" => {
            match command {
                cli::AdminCommands::CreateUser { username, password, role } => {
                    if let Err(e) = admin::create_user(&db, username, password, role) {
                        error!("Failed to create user: {}", e);
                    } else {
                        info!("User '{}' created successfully", username);
                    }
                }
                // Handle more admin commands
            }
        }
        Commands::Customer { command } if user.1.to_lowercase() == "customer" => {
            let account_id = user.0;
            match command {
                cli::CustomerCommands::Buy { asset, amount } => {
                    if let Err(e) = customer::buy(&db, account_id, asset, *amount) {
                        error!("Failed to execute buy: {}", e);
                    } else {
                        info!("Buy order executed successfully");
                    }
                }
                cli::CustomerCommands::Sell { asset, amount } => {
                    if let Err(e) = customer::sell(&db, account_id, asset, *amount) {
                        error!("Failed to execute sell: {}", e);
                    } else {
                        info!("Sell order executed successfully");
                    }
                }
                // Handle more customer commands
            }
        }
        Commands::Regulator { command } if user.1.to_lowercase() == "regulator" => {
            match command {
                cli::RegulatorCommands::ViewTransactions => {
                    match regulator::view_transactions(&db) {
                        Ok(transactions) => {
                            for tx in transactions {
                                println!("{:?}", tx);
                            }
                        }
                        Err(e) => error!("Failed to fetch transactions: {}", e),
                    }
                }
                cli::RegulatorCommands::ViewWallets => {
                    match regulator::view_wallets(&db) {
                        Ok(wallets) => {
                            for (account_id, balance) in wallets {
                                println!("Account ID: {}, Balance: {}", account_id, balance);
                            }
                        }
                        Err(e) => error!("Failed to fetch wallets: {}", e),
                    }
                }
                // Handle more regulator commands
            }
        }
        _ => {
            error!("Unauthorized or invalid command");
            process::exit(1);
        }
    }
}