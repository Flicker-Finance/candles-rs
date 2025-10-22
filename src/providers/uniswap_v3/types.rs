use strum::Display;

#[derive(Debug, Clone)]
pub struct ProcessedSwap {
    pub timestamp: i64,
    pub price: f64,
    pub volume_usd: f64,
}

#[derive(Debug, Clone, Copy, Display)]
pub enum Chain {
    Ethereum,
    Base,
    Bnb,
}

impl Chain {
    pub fn get_rpc_url(&self) -> String {
        let default_url = match self {
            Chain::Ethereum => "https://eth.llamarpc.com",
            Chain::Base => "https://base.llamarpc.com",
            Chain::Bnb => "https://binance.llamarpc.com",
        };

        let env_key = format!("{}_RPC_URL", self.to_string().to_uppercase());
        std::env::var(&env_key).unwrap_or_else(|_| default_url.to_string())
    }

    pub fn from_str(chain: &str) -> Option<Chain> {
        match chain.to_lowercase().as_str() {
            "ethereum" | "eth" | "mainnet" => Some(Chain::Ethereum),
            "base" => Some(Chain::Base),
            "bnb" | "bsc" | "binance" => Some(Chain::Bnb),
            _ => None,
        }
    }
}
