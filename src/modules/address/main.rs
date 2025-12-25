use crate::modules::address::{ethereum, solana};

pub fn parse_address(addr: &str) -> Option<String> {
    ethereum::main::parse_address(addr).or_else(|| solana::main::parse_address(addr))
}

pub fn is_valid_address(addr: &str) -> bool {
    parse_address(addr).is_some()
}
