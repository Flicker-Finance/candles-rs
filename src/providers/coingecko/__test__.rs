#[cfg(test)]
mod test {
    use crate::{
        connections::Connection,
        errors::CandlesError,
        providers::base::BaseConnection,
        providers::coingecko::main::CoinGecko,
        types::{Instrument, MarketType, Timeframe},
        utils::examine_candles,
    };

    #[tokio::test]
    async fn test_coingecko_ethereum_pool_15m() {
        let instrument = Instrument {
            asset_id: "ethereum_usdc_weth".to_owned(),
            pair: "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2_eth_0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640".to_owned(),
            asset_symbol: "WETH".to_owned(),
            connection: Connection::CoinGecko,
            market_type: MarketType::Spot,
            limit: None,
            start_time: None,
            end_time: None,
            timeframe: Timeframe::M15,
        };

        match CoinGecko::get_candles(instrument.clone()).await {
            Ok(result) => examine_candles(&result, instrument),
            Err(err) => panic!("Failed to fetch candles: {err}"),
        }
    }

    #[tokio::test]
    async fn test_coingecko_base_pool_1h() {
        let instrument = Instrument {
            asset_id: "base_test_pool".to_owned(),
            limit: None,
            start_time: None,
            end_time: None,
            pair: "0x3054e8f8fba3055a42e5f5228a2a4e2ab1326933_base_0xE1BeD6AAdBa5471700f16A47EEe2504346B724aD".to_owned(),
            asset_symbol: "WETH".to_owned(),
            connection: Connection::CoinGecko,
            market_type: MarketType::Spot,
            timeframe: Timeframe::H1,
        };

        match CoinGecko::get_candles(instrument.clone()).await {
            Ok(result) => examine_candles(&result, instrument),
            Err(err) => panic!("Failed to fetch candles: {err}"),
        }
    }

    #[tokio::test]
    async fn test_pagination_2000_candles() {
        let instrument = Instrument {
            asset_id: "ethereum_usdc_weth".to_owned(),
            pair: "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2_eth_0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640".to_owned(),
            asset_symbol: "WETH".to_owned(),
            limit: Some(2000),
            start_time: None,
            end_time: None,
            connection: Connection::CoinGecko,
            market_type: MarketType::Spot,
            timeframe: Timeframe::H1,
        };

        let candles = instrument.connection.get_candles(instrument.clone()).await.expect("Failed to fetch 2000 candles");

        assert!(candles.len() >= 2000, "Expected at least 2000 candles, got {}", candles.len());

        for i in 1..candles.len() {
            assert!(candles[i].timestamp > candles[i - 1].timestamp, "Candles not in ascending order at index {}", i);
        }
    }

    #[tokio::test]
    async fn test_coingecko_invalid_pair_format() {
        let instrument = Instrument {
            asset_id: "test".to_owned(),
            pair: "invalid_format_missing_underscore".to_owned(),
            asset_symbol: "TEST".to_owned(),
            connection: Connection::CoinGecko,
            limit: None,
            start_time: None,
            end_time: None,
            market_type: MarketType::Spot,
            timeframe: Timeframe::H1,
        };

        let result = CoinGecko::get_candles(instrument).await;

        match result {
            Ok(_) => panic!("Expected error for invalid pair format"),
            Err(err) => match err {
                CandlesError::InvalidAddress(_) => {}
                _ => panic!("Expected InvalidPairFormat error, got: {err:?}"),
            },
        }
    }

    #[tokio::test]
    async fn test_coingecko_malformed_pair() {
        let instrument = Instrument {
            asset_id: "test".to_owned(),
            pair: "nounderscore".to_owned(),
            asset_symbol: "TEST".to_owned(),
            connection: Connection::CoinGecko,
            limit: None,
            start_time: None,
            end_time: None,
            market_type: MarketType::Spot,
            timeframe: Timeframe::H1,
        };

        let result = CoinGecko::get_candles(instrument).await;

        assert!(result.is_err());
        assert!(matches!(result, Err(CandlesError::InvalidPoolFormat(_))));
    }
}
