#[cfg(test)]
mod test {
    use crate::{
        connections::Connection,
        errors::CandlesError,
        modules::chains::Chain,
        providers::base::BaseConnection,
        providers::moralis::main::Moralis,
        types::{Instrument, MarketType, Timeframe},
        utils::examine_candles,
    };

    #[tokio::test]
    #[ignore = "MORALIS_API_KEY needed"]
    async fn test_moralis_ethereum_usdc_weth_15m() {
        let instrument = Instrument {
            asset_id: "ethereum_usdc_weth".to_owned(),
            pair: "eth_0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640".to_owned(),
            connection: Connection::Moralis,
            market_type: MarketType::Spot,
            timeframe: Timeframe::M15,
        };

        match Moralis::get_candles(instrument).await {
            Ok(result) => examine_candles(&result),
            Err(err) => panic!("Failed to fetch candles: {err}"),
        }
    }

    #[tokio::test]
    #[ignore = "MORALIS_API_KEY needed"]
    async fn test_moralis_base_pool_1h() {
        let instrument = Instrument {
            asset_id: "base_test_pool".to_owned(),
            pair: "base_0xE1BeD6AAdBa5471700f16A47EEe2504346B724aD".to_owned(),
            connection: Connection::Moralis,
            market_type: MarketType::Spot,
            timeframe: Timeframe::H1,
        };

        match Moralis::get_candles(instrument).await {
            Ok(result) => examine_candles(&result),
            Err(err) => panic!("Failed to fetch candles: {err}"),
        }
    }

    #[tokio::test]
    #[ignore = "MORALIS_API_KEY needed"]
    async fn test_moralis_get_token_price_usdc() {
        let result = Moralis::get_token_price("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", Some(Chain::Ethereum)).await;

        match result {
            Ok(price_info) => {
                println!("Token: {} ({})", price_info.token_name, price_info.token_symbol);
                println!("Price: ${}", price_info.usd_price);
                println!("Decimals: {}", price_info.token_decimals);
                println!("24h Change: {:?}", price_info.percent_change_24h);
                println!("Exchange: {:?}", price_info.exchange_name);

                assert_eq!(price_info.token_symbol, "USDC");
                assert!(price_info.usd_price > 0.0);
            }
            Err(err) => panic!("Failed to fetch token price: {err}"),
        }
    }

    #[tokio::test]
    #[ignore = "MORALIS_API_KEY needed"]
    async fn test_moralis_get_token_price_weth() {
        let result = Moralis::get_token_price("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2", None).await;

        match result {
            Ok(price_info) => {
                println!("Token: {} ({})", price_info.token_name, price_info.token_symbol);
                println!("Price: ${}", price_info.usd_price);
                println!("Native Price: {:?}", price_info.native_price);

                assert_eq!(price_info.token_symbol, "WETH");
                assert!(price_info.usd_price > 0.0);
            }
            Err(err) => panic!("Failed to fetch token price: {err}"),
        }
    }

    #[tokio::test]
    #[ignore = "MORALIS_API_KEY needed"]
    async fn test_moralis_get_token_price_base_chain() {
        let result = Moralis::get_token_price("0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913", Some(Chain::Base)).await;

        match result {
            Ok(price_info) => {
                println!("Token: {} ({})", price_info.token_name, price_info.token_symbol);
                println!("Price: ${}", price_info.usd_price);

                assert!(price_info.usd_price > 0.0);
            }
            Err(err) => panic!("Failed to fetch token price on Base: {err}"),
        }
    }

    #[tokio::test]
    async fn test_moralis_get_token_price_invalid_address() {
        let result = Moralis::get_token_price("0xinvalid", Some(Chain::Ethereum)).await;

        match result {
            Ok(_) => panic!("Expected error for invalid address"),
            Err(err) => match err {
                CandlesError::InvalidAddress(addr) => {
                    assert_eq!(addr, "0xinvalid");
                }
                _ => panic!("Expected InvalidAddress error, got: {:?}", err),
            },
        }
    }

    #[tokio::test]
    async fn test_moralis_get_token_price_malformed_address() {
        let result = Moralis::get_token_price("not_an_address", None).await;

        assert!(result.is_err());
        assert!(matches!(result, Err(CandlesError::InvalidAddress(_))));
    }
}
