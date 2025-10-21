use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Debug, Deserialize, Serialize)]
pub struct GraphQLResponse {
    pub data: SwapsData,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SwapsData {
    pub swaps: Vec<Swap>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Swap {
    pub timestamp: String,
    #[serde(rename = "amount0")]
    pub amount_0: String,
    #[serde(rename = "amount1")]
    pub amount_1: String,
    #[serde(rename = "amountUSD")]
    pub amount_usd: String,
    #[serde(rename = "sqrtPriceX96")]
    pub sqrt_price_x96: String,
    pub tick: String,
}

#[derive(Debug, Clone)]
pub struct ProcessedSwap {
    pub timestamp: i64,
    pub price: f64,
    pub volume_usd: f64,
}

/// Represents the supported chains for Uniswap V3
#[derive(Debug, Clone, Copy, Display)]
pub enum Chain {
    Ethereum,
    Polygon,
    Arbitrum,
    Base,
    Bnb,
}

impl Chain {
    /// Get the subgraph deployment ID for The Graph Network
    pub fn get_subgraph_id(&self) -> &'static str {
        match self {
            // Official Uniswap V3 subgraph deployment IDs on The Graph Network
            Chain::Ethereum => "5zvR82QoaXYFyDEKLZ9t6v9adgnptxYpKpSbxtgVENFV",
            Chain::Polygon => "3hCPRGf4z88VC5rsBKU5AA9FBBq5nF3jbKJG7VZCbhjm",
            Chain::Arbitrum => "FbCGRftH4a3yZugY7TnbYgPJVEv2LvMT6oF1fxPe9aJM",
            Chain::Base => "HMuAwufqZ1YCRmzL2SfHTVkzZovC9VL2UAKhjvRqKiR1",
            Chain::Bnb => "G5MUbSBM7Nsrm9tH2tGQUiAF4SZDGf2qeo1xPLYjKr7K",
        }
    }

    pub fn get_subgraph_url(&self, api_key: &str) -> String {
        format!("https://gateway.thegraph.com/api/{}/subgraphs/id/{}", api_key, self.get_subgraph_id())
    }

    pub fn from_str(chain: &str) -> Option<Chain> {
        match chain.to_lowercase().as_str() {
            "ethereum" | "eth" | "mainnet" => Some(Chain::Ethereum),
            "polygon" | "matic" => Some(Chain::Polygon),
            "arbitrum" | "arb" => Some(Chain::Arbitrum),
            "base" => Some(Chain::Base),
            "bnb" | "bsc" | "binance" => Some(Chain::Bnb),
            _ => None,
        }
    }
}
