use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::{
    errors::CandlesError,
    providers::base::BaseConnection,
    providers::moralis::types::MoralisOhlcvResponse,
    types::{Candle, Instrument, Timeframe},
};

pub struct Moralis;

impl Moralis {
    fn parse_chain_and_address(pair: &str) -> Result<(String, String), CandlesError> {
        let parts: Vec<&str> = pair.split('_').collect();
        if parts.len() != 2 {
            return Err(CandlesError::Other(format!("Invalid pair format. Expected 'chain_pairAddress', got: {pair}")));
        }

        let chain = parts[0].to_lowercase();
        let pair_address = parts[1].to_lowercase();

        Ok((chain, pair_address))
    }

    fn get_moralis_timeframe(timeframe: &Timeframe) -> Result<&'static str, CandlesError> {
        match timeframe {
            Timeframe::M3 => Err(CandlesError::Other("3m Timeframe is not available for Moralis".to_string())),
            Timeframe::M5 => Ok("5min"),
            Timeframe::M15 => Ok("15min"),
            Timeframe::M30 => Ok("30min"),
            Timeframe::H1 => Ok("1h"),
            Timeframe::H4 => Ok("4h"),
            Timeframe::D1 => Ok("1d"),
            Timeframe::W1 => Err(CandlesError::Other("1w Timeframe is not available for Moralis".to_string())),
            Timeframe::MN1 => Err(CandlesError::Other("1M Timeframe is not available for Moralis".to_string())),
        }
    }
}

#[async_trait]
impl BaseConnection for Moralis {
    async fn get_candles(instrument: Instrument) -> Result<Vec<Candle>, CandlesError> {
        let api_key = std::env::var("MORALIS_API_KEY").map_err(|_| CandlesError::Other("MORALIS_API_KEY environment variable not set".to_string()))?;

        let (chain, pair_address) = Self::parse_chain_and_address(&instrument.pair)?;
        let timeframe = Self::get_moralis_timeframe(&instrument.timeframe)?;

        // Calculate date range - get data from 30 days ago to now
        let now = Utc::now();
        let from_date = now - chrono::Duration::days(30);

        // Format dates in the required format: 2025-01-01T10:00:00.000
        let from_date_str = from_date.format("%Y-%m-%dT%H:%M:%S%.3f").to_string();
        let to_date_str = now.format("%Y-%m-%dT%H:%M:%S%.3f").to_string();

        let url = format!("https://deep-index.moralis.io/api/v2.2/pairs/{pair_address}/ohlcv");

        let client = reqwest::Client::new();
        let response_api: serde_json::Value = client
            .get(&url)
            .header("X-API-Key", api_key)
            .query(&[
                ("chain", chain.as_str()),
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
        println!("response_api {response_api}");

        let response = serde_json::from_value::<MoralisOhlcvResponse>(response_api).map_err(|err| CandlesError::Other(err.to_string()))?;

        let mut candles: Vec<Candle> = response
            .result
            .into_iter()
            .map(|candle| {
                // Parse timestamp (ISO 8601 format) to milliseconds
                let timestamp_ms = DateTime::parse_from_rfc3339(&candle.timestamp)
                    .map_err(|e| CandlesError::Other(format!("Failed to parse timestamp: {e}")))?
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
