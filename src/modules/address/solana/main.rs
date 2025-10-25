use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

pub fn parse_address(addr: &str) -> Option<String> {
    Pubkey::from_str(addr).map(|f| f.to_string()).ok()
}

pub fn is_valid_address(addr: &str) -> bool {
    parse_address(addr).is_some()
}
