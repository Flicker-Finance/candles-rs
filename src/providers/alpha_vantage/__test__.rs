#[cfg(test)]
mod test {
    use crate::{
        connections::Connection,
        errors::CandlesError,
        providers::alpha_vantage::main::AlphaVantage,
        providers::base::BaseConnection,
        types::{Instrument, MarketType, Timeframe},
    };
    use std::time::Duration;
    use tokio::time::sleep;

    fn load_env() {
        let _ = dotenvy::dotenv();
    }

    // AlphaVantage free tier has strict rate limits (5 requests/min)
    // Different delays to stagger parallel test execution
    async fn rate_limit_delay(secs: u64) {
        sleep(Duration::from_secs(secs)).await;
    }

    fn check_candles(candles: &[crate::types::Candle]) {
        assert!(!candles.is_empty(), "Candles array is empty");

        // Check all candles are in ascending order
        for i in 1..candles.len() {
            assert!(candles[i].timestamp > candles[i - 1].timestamp, "Candles are not in ascending order");
        }

        // Basic validation on last candle
        let candle = candles.last().unwrap();
        assert!(candle.high >= candle.low, "High should be >= low");
        assert!(candle.close > 0.0, "Close price should be positive");
        assert!(candle.volume >= 0.0, "Volume should be non-negative");
    }

    #[tokio::test]
    async fn test_alpha_vantage_daily() {
        load_env();
        rate_limit_delay(0).await;
        let instrument = Instrument {
            asset_id: "NVDA".to_owned(),
            pair: "NVDA".to_owned(),
            connection: Connection::AlphaVantage,
            market_type: MarketType::Spot,
            limit: None,
            timeframe: Timeframe::D1,
        };

        match AlphaVantage::get_candles(instrument).await {
            Ok(result) => {
                check_candles(&result);
                println!("Fetched {} daily candles for NVDA", result.len());
            }
            Err(err) => panic!("Failed to fetch candles: {err}"),
        }
    }

    #[tokio::test]
    async fn test_alpha_vantage_weekly() {
        load_env();
        rate_limit_delay(15).await;
        let instrument = Instrument {
            asset_id: "AAPL".to_owned(),
            pair: "AAPL".to_owned(),
            connection: Connection::AlphaVantage,
            market_type: MarketType::Spot,
            limit: None,
            timeframe: Timeframe::W1,
        };

        match AlphaVantage::get_candles(instrument).await {
            Ok(result) => {
                check_candles(&result);
                println!("Fetched {} weekly candles for AAPL", result.len());
            }
            Err(err) => panic!("Failed to fetch candles: {err}"),
        }
    }

    #[tokio::test]
    #[ignore] // Intraday requires premium API key
    async fn test_alpha_vantage_intraday_5min() {
        load_env();
        rate_limit_delay(30).await;
        let instrument = Instrument {
            asset_id: "MSFT".to_owned(),
            pair: "MSFT".to_owned(),
            connection: Connection::AlphaVantage,
            market_type: MarketType::Spot,
            limit: None,
            timeframe: Timeframe::M5,
        };

        match AlphaVantage::get_candles(instrument).await {
            Ok(result) => {
                check_candles(&result);
                println!("Fetched {} 5min candles for MSFT", result.len());
            }
            Err(err) => panic!("Failed to fetch candles: {err}"),
        }
    }

    #[tokio::test]
    async fn test_alpha_vantage_invalid_symbol() {
        load_env();
        rate_limit_delay(45).await;
        let instrument = Instrument {
            asset_id: "INVALID_SYMBOL_XYZ123".to_owned(),
            pair: "INVALID_SYMBOL_XYZ123".to_owned(),
            connection: Connection::AlphaVantage,
            market_type: MarketType::Spot,
            limit: None,
            timeframe: Timeframe::D1,
        };

        let result = AlphaVantage::get_candles(instrument).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_alpha_vantage_unsupported_timeframe() {
        let instrument = Instrument {
            asset_id: "NVDA".to_owned(),
            pair: "NVDA".to_owned(),
            connection: Connection::AlphaVantage,
            market_type: MarketType::Spot,
            limit: None,
            timeframe: Timeframe::M3,
        };

        let result = AlphaVantage::get_candles(instrument).await;

        match result {
            Ok(_) => panic!("Expected error for unsupported timeframe"),
            Err(err) => match err {
                CandlesError::UnsupportedTimeframe { .. } => {}
                _ => panic!("Expected UnsupportedTimeframe error, got: {err:?}"),
            },
        }
    }
}
