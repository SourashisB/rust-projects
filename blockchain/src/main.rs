mod blockchain;
mod block;
mod transaction;
mod wallet;
mod cli;

use structopt::StructOpt;
use crate::blockchain::Blockchain;
use crate::cli::Cli;

fn main() {
    let mut blockchain = Blockchain::new();
    let cli = Cli::from_args();

    match cli {
        Cli::CreateWallet => {
            let wallet = blockchain.create_wallet();
            println!("New wallet created: {}", wallet.address);
        }
        Cli::GetBalance { address } => {
            match blockchain.get_wallet_balance(&address) {
                Ok(balance) => println!("Balance of wallet {}: {}", address, balance),
                Err(e) => println!("Error: {}", e),
            }
        }
        Cli::SendCoins { from, to, amount } => {
            match blockchain.add_transaction(from, to, amount) {
                Ok(_) => println!("Transaction added successfully"),
                Err(e) => println!("Error: {}", e),
            }
        }
        Cli::Mine => {
            let block = blockchain.mine();
            println!("New block mined: {:#?}", block);
        }
        Cli::PrintChain => {
            let chain = blockchain.get_chain();
            println!("{:#?}", chain);
        }
    }
}