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

/// Checks if a string is a valid Ethereum address
pub fn is_valid_address(addr: &str) -> bool {
    Address::from_str(addr).is_ok()
}

/// Parses a string into an Address, returning None if invalid
pub fn parse_address(addr: &str) -> Option<Address> {
    Address::from_str(addr).ok()
}

/// Fetches ERC20 token information (name, symbol, decimals) from the blockchain
pub async fn get_token_info(chain: Chain, address: &str) -> Result<TokenInfo, CandlesError> {
    let token_address = Address::from_str(address).map_err(|_| CandlesError::InvalidAddress(address.to_string()))?;

    let rpc_url = chain.get_rpc_url();
    let provider = ProviderBuilder::new().on_http(rpc_url.parse().map_err(|e| CandlesError::RpcError(format!("Invalid RPC URL: {e}")))?);

    // Fetch all token info in parallel
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
    // ERC20 name() function signature: 0x06fdde03
    let name_selector = keccak256("name()");
    let calldata = name_selector[0..4].to_vec();

    let tx = alloy::rpc::types::TransactionRequest::default().to(token_address).input(calldata.into());

    let result = provider.call(&tx).await.map_err(|e| CandlesError::RpcError(format!("Failed to call name(): {e}")))?;

    // Decode string from ABI encoded response
    decode_string(&result).ok_or_else(|| CandlesError::InvalidBlockchainData(format!("Failed to decode token name for address {token_address}")))
}

async fn get_token_symbol(provider: &impl Provider<Http<reqwest::Client>>, token_address: Address) -> Result<String, CandlesError> {
    // ERC20 symbol() function signature: 0x95d89b41
    let symbol_selector = keccak256("symbol()");
    let calldata = symbol_selector[0..4].to_vec();

    let tx = alloy::rpc::types::TransactionRequest::default().to(token_address).input(calldata.into());

    let result = provider.call(&tx).await.map_err(|e| CandlesError::RpcError(format!("Failed to call symbol(): {e}")))?;

    // Decode string from ABI encoded response
    decode_string(&result).ok_or_else(|| CandlesError::InvalidBlockchainData(format!("Failed to decode token symbol for address {token_address}")))
}

async fn get_token_decimals(provider: &impl Provider<Http<reqwest::Client>>, token_address: Address) -> Result<u8, CandlesError> {
    // ERC20 decimals() function signature: 0x313ce567
    let decimals_selector = keccak256("decimals()");
    let calldata = decimals_selector[0..4].to_vec();

    let tx = alloy::rpc::types::TransactionRequest::default().to(token_address).input(calldata.into());

    let result = provider.call(&tx).await.map_err(|e| CandlesError::RpcError(format!("Failed to call decimals(): {e}")))?;

    if result.len() < 32 {
        return Err(CandlesError::InvalidBlockchainData("Invalid decimals response".to_string()));
    }

    // decimals returns uint8, but it's padded to 32 bytes
    Ok(result[31])
}

/// Decodes an ABI-encoded string from contract call response
fn decode_string(data: &[u8]) -> Option<String> {
    if data.len() < 64 {
        return None;
    }

    // ABI encoding for string:
    // [0..32]   - offset (should be 32)
    // [32..64]  - length
    // [64..]    - actual string data

    let length = u64::from_be_bytes(data[56..64].try_into().ok()?) as usize;

    if data.len() < 64 + length {
        return None;
    }

    let string_bytes = &data[64..64 + length];
    String::from_utf8(string_bytes.to_vec()).ok()
}
