use crate::{errors::CandlesError, modules::chains::Chain};
use alloy::primitives::Address;
use std::str::FromStr;

pub struct Pool {
    pub token_address: Address,
    pub chain: Chain,
    pub pool_address: Address,
    pub inverted: bool,
}

impl TryFrom<String> for Pool {
    type Error = CandlesError;

    fn try_from(pair: String) -> Result<Self, Self::Error> {
        let parts: Vec<&str> = pair.split('_').collect();
        if parts.len() < 3 {
            return Err(CandlesError::InvalidPoolFormat(format!(
                "Expected 'tokenAddress_chain_poolAddress' or 'tokenAddress_chain_poolAddress_inverted', got: {pair}"
            )));
        }

        let token_address = Address::from_str(parts[0]).map_err(|_err| CandlesError::InvalidAddress(parts[0].to_string()))?;
        let chain = Chain::try_from(parts[1])?;
        let pool_address = Address::from_str(parts[2]).map_err(|_err| CandlesError::InvalidAddress(parts[2].to_string()))?;
        let inverted = parts.iter().skip(3).any(|&s| s == "inverted");

        Ok(Self {
            token_address,
            chain,
            pool_address,
            inverted,
        })
    }
}
