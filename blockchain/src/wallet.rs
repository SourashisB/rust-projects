use rand::Rng;

#[derive(Debug, Clone)]
pub struct Wallet {
    pub address: String,
    pub balance: f64,
}

impl Wallet {
    pub fn new() -> Self {
        Wallet {
            address: Wallet::generate_address(),
            balance: 0.0,
        }
    }

    fn generate_address() -> String {
        let mut rng = rand::thread_rng();
        let wallet_bytes: Vec<u8> = (0..20).map(|_| rng.gen()).collect();
        hex::encode(wallet_bytes)
    }
}