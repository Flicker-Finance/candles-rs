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
            pair: "eth_0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640".to_owned(),
            connection: Connection::CoinGecko,
            market_type: MarketType::Spot,
            timeframe: Timeframe::M15,
        };

        match CoinGecko::get_candles(instrument).await {
            Ok(result) => examine_candles(&result),
            Err(err) => panic!("Failed to fetch candles: {err}"),
        }
    }

    #[tokio::test]
    async fn test_coingecko_base_pool_1h() {
        let instrument = Instrument {
            asset_id: "base_test_pool".to_owned(),
            pair: "base_0xE1BeD6AAdBa5471700f16A47EEe2504346B724aD".to_owned(),
            connection: Connection::CoinGecko,
            market_type: MarketType::Spot,
            timeframe: Timeframe::H1,
        };

        match CoinGecko::get_candles(instrument).await {
            Ok(result) => examine_candles(&result),
            Err(err) => panic!("Failed to fetch candles: {err}"),
        }
    }

    #[tokio::test]
    async fn test_coingecko_invalid_pair_format() {
        let instrument = Instrument {
            asset_id: "test".to_owned(),
            pair: "invalid_format_missing_underscore".to_owned(),
            connection: Connection::CoinGecko,
            market_type: MarketType::Spot,
            timeframe: Timeframe::H1,
        };

        let result = CoinGecko::get_candles(instrument).await;

        match result {
            Ok(_) => panic!("Expected error for invalid pair format"),
            Err(err) => match err {
                CandlesError::InvalidPairFormat(_) => {}
                _ => panic!("Expected InvalidPairFormat error, got: {:?}", err),
            },
        }
    }

    #[tokio::test]
    async fn test_coingecko_malformed_pair() {
        let instrument = Instrument {
            asset_id: "test".to_owned(),
            pair: "nounderscore".to_owned(),
            connection: Connection::CoinGecko,
            market_type: MarketType::Spot,
            timeframe: Timeframe::H1,
        };

        let result = CoinGecko::get_candles(instrument).await;

        assert!(result.is_err());
        assert!(matches!(result, Err(CandlesError::InvalidPairFormat(_))));
    }
}
