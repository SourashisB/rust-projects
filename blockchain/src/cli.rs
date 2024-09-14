use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "simple_blockchain")]
pub enum Cli {
    CreateWallet,
    GetBalance {
        #[structopt(short, long)]
        address: String,
    },
    SendCoins {
        #[structopt(short, long)]
        from: String,
        #[structopt(short, long)]
        to: String,
        #[structopt(short, long)]
        amount: f64,
    },
    Mine,
    PrintChain,
}