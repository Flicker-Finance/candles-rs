use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::{
    errors::CandlesError,
    modules::pairs::parse_pair,
    providers::{base::BaseConnection, moralis::types::MoralisOhlcvResponse},
    types::{Candle, Instrument, Timeframe},
};

pub struct Moralis;

impl Moralis {
    fn get_moralis_timeframe(timeframe: &Timeframe) -> Result<&'static str, CandlesError> {
        match timeframe {
            Timeframe::M3 => Err(CandlesError::UnsupportedTimeframe {
                timeframe: "3m".to_string(),
                provider: "Moralis".to_string(),
            }),
            Timeframe::M5 => Ok("5min"),
            Timeframe::M15 => Ok("15min"),
            Timeframe::M30 => Ok("30min"),
            Timeframe::H1 => Ok("1h"),
            Timeframe::H4 => Ok("4h"),
            Timeframe::D1 => Ok("1d"),
            Timeframe::W1 => Err(CandlesError::UnsupportedTimeframe {
                timeframe: "1w".to_string(),
                provider: "Moralis".to_string(),
            }),
            Timeframe::MN1 => Err(CandlesError::UnsupportedTimeframe {
                timeframe: "1M".to_string(),
                provider: "Moralis".to_string(),
            }),
        }
    }
}

#[async_trait]
impl BaseConnection for Moralis {
    async fn get_candles(instrument: Instrument) -> Result<Vec<Candle>, CandlesError> {
        let api_key = std::env::var("MORALIS_API_KEY").map_err(|_| CandlesError::MissingEnvVar("MORALIS_API_KEY".to_string()))?;

        let (chain, pair_address, _inverted) = parse_pair(&instrument.pair)?;
        let timeframe = Self::get_moralis_timeframe(&instrument.timeframe)?;

        // Calculate date range - get data from 30 days ago to now
        let now = Utc::now();
        let from_date = now - chrono::Duration::days(30);

        // Format dates in the required format: 2025-01-01T10:00:00.000
        let from_date_str = from_date.format("%Y-%m-%dT%H:%M:%S%.3f").to_string();
        let to_date_str = now.format("%Y-%m-%dT%H:%M:%S%.3f").to_string();

        let url = format!("https://deep-index.moralis.io/api/v2.2/pairs/{pair_address}/ohlcv");

        let client = reqwest::Client::new();
        let response: MoralisOhlcvResponse = client
            .get(&url)
            .header("X-API-Key", api_key)
            .query(&[
                ("chain", chain.as_str_moralis()),
                ("timeframe", timeframe),
                ("currency", "usd"),
                ("fromDate", from_date_str.as_str()),
                ("toDate", to_date_str.as_str()),
                ("limit", "300"),
            ])
            .send()
            .await?
            .json()
            .await?;

        let mut candles: Vec<Candle> = response
            .result
            .into_iter()
            .map(|candle| {
                // Parse timestamp (ISO 8601 format) to milliseconds
                let timestamp_ms = DateTime::parse_from_rfc3339(&candle.timestamp)
                    .map_err(|e| CandlesError::ParseError {
                        field: "timestamp".to_string(),
                        message: e.to_string(),
                    })?
                    .timestamp_millis();

                Ok(Candle {
                    timestamp: timestamp_ms,
                    open: candle.open,
                    high: candle.high,
                    low: candle.low,
                    close: candle.close,
                    volume: candle.volume,
                })
            })
            .collect::<Result<Vec<Candle>, CandlesError>>()?;

        // Moralis returns candles in descending order (newest first), reverse to ascending (oldest first)
        candles.reverse();

        Ok(candles)
    }
}
