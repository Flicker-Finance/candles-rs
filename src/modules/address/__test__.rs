#[cfg(test)]
mod tests {
    use crate::modules::{
        address::main::{get_token_info, is_valid_address, parse_address},
        chains::Chain,
    };

    #[test]
    fn test_valid_address() {
        // Valid checksummed address
        assert!(is_valid_address("0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed"));

        // Valid lowercase address
        assert!(is_valid_address("0x5aaeb6053f3e94c9b9a09f33669435e7ef1beaed"));

        // Valid uppercase address
        assert!(is_valid_address("0x5AAEB6053F3E94C9B9A09F33669435E7EF1BEAED"));
    }

    #[test]
    fn test_invalid_address() {
        // Too short
        assert!(!is_valid_address("0x123"));

        // Invalid characters
        assert!(!is_valid_address("0xGGGeb6053F3E94C9b9A09f33669435E7Ef1BeAed"));

        // Empty string
        assert!(!is_valid_address(""));

        // Invalid length (missing one character)
        assert!(!is_valid_address("0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAe"));

        // Too long
        assert!(!is_valid_address("0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAedAA"));
    }

    #[test]
    fn test_parse_address() {
        let valid = "0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed";
        assert!(parse_address(valid).is_some());

        let invalid = "0x123";
        assert!(parse_address(invalid).is_none());
    }

    // Integration tests require RPC access
    #[tokio::test]
    async fn test_get_token_info_usdc() {
        // USDC on Ethereum mainnet

        match get_token_info(Chain::Ethereum, "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48").await {
            Ok(info) => {
                assert_eq!(info.symbol, "USDC");
                assert_eq!(info.decimals, 6);
                assert!(info.name.contains("USD"));
            }
            Err(err) => panic!("{err}"),
        }
    }

    #[tokio::test]
    async fn test_get_token_info_invalid() {
        // Invalid address
        let result = get_token_info(Chain::Ethereum, "0x123").await;
        assert!(result.is_err());
    }
}
