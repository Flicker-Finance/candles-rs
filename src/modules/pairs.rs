use std::str::FromStr;

use alloy::primitives::Address;

use crate::{errors::CandlesError, modules::chains::Chain};

pub fn parse_pair(pair: &str) -> Result<(Chain, String, bool), CandlesError> {
    let parts: Vec<&str> = pair.split('_').collect();
    if parts.len() < 2 {
        return Err(CandlesError::InvalidPairFormat(format!(
            "Expected 'chain_poolAddress' or 'chain_poolAddress_inverted', got: {pair}"
        )));
    }

    let chain = Chain::try_from(parts[0])?;

    let pool_address = Address::from_str(parts[1]).map_err(|_err| CandlesError::InvalidAddress(parts[1].to_string()))?;

    let invert_price = parts.iter().skip(2).any(|&s| s == "inverted");

    Ok((chain, pool_address.to_string(), invert_price))
}
