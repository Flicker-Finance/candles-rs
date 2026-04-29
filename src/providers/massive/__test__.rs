#[cfg(test)]
mod test {
    use crate::{
        connections::Connection,
        errors::CandlesError,
        providers::base::BaseConnection,
        providers::massive::main::Massive,
        types::{Instrument, MarketType, Timeframe},
    };
    use std::time::Duration;
    use tokio::time::sleep;

    fn load_env() {
        let _ = dotenvy::dotenv();
    }

    // Massive free plan is 5 req/min. Stagger parallel tests.
    async fn rate_limit_delay(secs: u64) {
        sleep(Duration::from_secs(secs)).await;
    }

    fn check_candles(candles: &[crate::types::Candle]) {
        assert!(!candles.is_empty(), "Candles array is empty");

        for i in 1..candles.len() {
            assert!(candles[i].timestamp > candles[i - 1].timestamp, "Candles are not in ascending order");
        }

        let candle = candles.last().unwrap();
        assert!(candle.high >= candle.low, "High should be >= low");
        assert!(candle.close > 0.0, "Close price should be positive");
        assert!(candle.volume >= 0.0, "Volume should be non-negative");
    }

    #[tokio::test]
    async fn test_massive_daily() {
        load_env();
        rate_limit_delay(0).await;
        let instrument = Instrument {
            asset_id: "aapl".to_owned(),
            asset_symbol: "AAPL".to_owned(),
            pair: "AAPL-USD".to_owned(),
            connection: Connection::Massive,
            market_type: MarketType::Spot,
            limit: Some(100),
            start_time: None,
            end_time: None,
            timeframe: Timeframe::D1,
        };

        match Massive::get_candles(instrument).await {
            Ok(result) => {
                check_candles(&result);
                println!("Fetched {} daily candles for AAPL", result.len());
            }
            Err(err) => panic!("Failed to fetch candles: {err}"),
        }
    }

    #[tokio::test]
    async fn test_massive_weekly() {
        load_env();
        rate_limit_delay(15).await;
        let instrument = Instrument {
            asset_id: "msft".to_owned(),
            asset_symbol: "MSFT".to_owned(),
            pair: "MSFT-USD".to_owned(),
            connection: Connection::Massive,
            market_type: MarketType::Spot,
            limit: Some(100),
            start_time: None,
            end_time: None,
            timeframe: Timeframe::W1,
        };

        match Massive::get_candles(instrument).await {
            Ok(result) => {
                check_candles(&result);
                println!("Fetched {} weekly candles for MSFT", result.len());
            }
            Err(err) => panic!("Failed to fetch candles: {err}"),
        }
    }

    #[tokio::test]
    async fn test_massive_intraday_15min() {
        load_env();
        rate_limit_delay(30).await;
        let instrument = Instrument {
            asset_id: "nvda".to_owned(),
            asset_symbol: "NVDA".to_owned(),
            pair: "NVDA-USD".to_owned(),
            connection: Connection::Massive,
            market_type: MarketType::Spot,
            limit: Some(100),
            start_time: None,
            end_time: None,
            timeframe: Timeframe::M15,
        };

        match Massive::get_candles(instrument).await {
            Ok(result) => {
                check_candles(&result);
                println!("Fetched {} 15min candles for NVDA", result.len());
            }
            Err(err) => panic!("Failed to fetch candles: {err}"),
        }
    }

    #[tokio::test]
    async fn test_massive_unsupported_timeframe() {
        let instrument = Instrument {
            asset_id: "aapl".to_owned(),
            asset_symbol: "AAPL".to_owned(),
            pair: "AAPL-USD".to_owned(),
            connection: Connection::Massive,
            market_type: MarketType::Spot,
            limit: None,
            start_time: None,
            end_time: None,
            timeframe: Timeframe::M3,
        };

        let result = Massive::get_candles(instrument).await;

        match result {
            Ok(_) => panic!("Expected error for unsupported timeframe"),
            Err(err) => match err {
                CandlesError::UnsupportedTimeframe { .. } => {}
                _ => panic!("Expected UnsupportedTimeframe error, got: {err:?}"),
            },
        }
    }
}
