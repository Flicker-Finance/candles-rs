use strum::{Display, EnumString};

#[derive(Debug, Clone, Copy, Display, EnumString)]
pub enum Chain {
    #[strum(serialize = "ethereum", serialize = "eth", serialize = "mainnet")]
    Ethereum,

    #[strum(serialize = "base")]
    Base,

    #[strum(serialize = "bnb", serialize = "bsc", serialize = "binance")]
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
}
