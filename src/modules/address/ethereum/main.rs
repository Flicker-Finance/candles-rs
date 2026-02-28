use alloy::primitives::{Address, keccak256};
use alloy::providers::{Provider, ProviderBuilder};
use alloy::transports::http::Http;
use std::str::FromStr;

use crate::errors::CandlesError;
use crate::modules::chains::Chain;

#[derive(Debug, Clone)]
pub struct TokenInfo {
    pub address: Address,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
}

pub fn parse_address(addr: &str) -> Option<String> {
    Address::from_str(addr).map(|f| f.to_string()).ok()
}

pub fn is_valid_address(addr: &str) -> bool {
    parse_address(addr).is_some()
}

pub async fn get_token_info(chain: Chain, address: &str) -> Result<TokenInfo, CandlesError> {
    let token_address = Address::from_str(address).map_err(|_| CandlesError::InvalidAddress(address.to_string()))?;

    let rpc_url = chain.get_rpc_url();
    let provider = ProviderBuilder::new().on_http(rpc_url.parse().map_err(|e| CandlesError::RpcError(format!("Invalid RPC URL: {e}")))?);

    let name_future = get_token_name(&provider, token_address);
    let symbol_future = get_token_symbol(&provider, token_address);
    let decimals_future = get_token_decimals(&provider, token_address);

    let (name, symbol, decimals) = tokio::try_join!(name_future, symbol_future, decimals_future)?;

    Ok(TokenInfo {
        address: token_address,
        name,
        symbol,
        decimals,
    })
}

async fn get_token_name(provider: &impl Provider<Http<reqwest::Client>>, token_address: Address) -> Result<String, CandlesError> {
    let name_selector = keccak256("name()");
    let calldata = name_selector[0..4].to_vec();

    let tx = alloy::rpc::types::TransactionRequest::default().to(token_address).input(calldata.into());

    let result = provider.call(&tx).await.map_err(|e| CandlesError::RpcError(format!("Failed to call name(): {e}")))?;

    decode_string(&result).ok_or_else(|| CandlesError::InvalidBlockchainData(format!("Failed to decode token name for address {token_address}")))
}

async fn get_token_symbol(provider: &impl Provider<Http<reqwest::Client>>, token_address: Address) -> Result<String, CandlesError> {
    let symbol_selector = keccak256("symbol()");
    let calldata = symbol_selector[0..4].to_vec();

    let tx = alloy::rpc::types::TransactionRequest::default().to(token_address).input(calldata.into());

    let result = provider.call(&tx).await.map_err(|e| CandlesError::RpcError(format!("Failed to call symbol(): {e}")))?;

    decode_string(&result).ok_or_else(|| CandlesError::InvalidBlockchainData(format!("Failed to decode token symbol for address {token_address}")))
}

async fn get_token_decimals(provider: &impl Provider<Http<reqwest::Client>>, token_address: Address) -> Result<u8, CandlesError> {
    let decimals_selector = keccak256("decimals()");
    let calldata = decimals_selector[0..4].to_vec();

    let tx = alloy::rpc::types::TransactionRequest::default().to(token_address).input(calldata.into());

    let result = provider.call(&tx).await.map_err(|e| CandlesError::RpcError(format!("Failed to call decimals(): {e}")))?;

    if result.len() < 32 {
        return Err(CandlesError::InvalidBlockchainData("Invalid decimals response".to_string()));
    }

    Ok(result[31])
}

fn decode_string(data: &[u8]) -> Option<String> {
    if data.len() < 64 {
        return None;
    }

    let length = u64::from_be_bytes(data[56..64].try_into().ok()?) as usize;

    if data.len() < 64 + length {
        return None;
    }

    let string_bytes = &data[64..64 + length];
    String::from_utf8(string_bytes.to_vec()).ok()
}
