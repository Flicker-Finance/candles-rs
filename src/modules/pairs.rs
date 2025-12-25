use crate::{
    errors::CandlesError,
    modules::{address::main::parse_address, chains::Chain},
};

pub struct Pool {
    pub token_address: String,
    pub chain: Chain,
    pub pool_address: String,
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

        let token_address = parse_address(parts[0]).ok_or(CandlesError::InvalidAddress(parts[0].to_string()))?;
        let chain = Chain::try_from(parts[1])?;
        let pool_address = parse_address(parts[2]).ok_or(CandlesError::InvalidAddress(parts[2].to_string()))?;
        let inverted = parts.iter().skip(3).any(|&s| s == "inverted");

        Ok(Self {
            token_address,
            chain,
            pool_address,
            inverted,
        })
    }
}
