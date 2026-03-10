use async_trait::async_trait;
use serde_json::Value;

use crate::{
    errors::CandlesError,
    providers::base::BaseConnection,
    types::{Candle, Instrument, MarketType, Timeframe},
    utils::parse_string_to_f64,
};

pub struct FreeDX;

#[async_trait]
impl BaseConnection for FreeDX {
    async fn get_candles(instrument: Instrument) -> Result<Vec<Candle>, CandlesError> {
        let resolution = match instrument.timeframe {
            Timeframe::M3 => {
                return Err(CandlesError::UnsupportedTimeframe {
                    timeframe: "3m".to_string(),
                    provider: "FreeDX".to_string(),
                });
            }
            Timeframe::M5 => "5",
            Timeframe::M15 => "15",
            Timeframe::M30 => "30",
            Timeframe::H1 => "60",
            Timeframe::H4 => "240",
            Timeframe::D1 | Timeframe::W1 | Timeframe::MN1 => {
                return Err(CandlesError::UnsupportedTimeframe {
                    timeframe: format!("{:?}", instrument.timeframe),
                    provider: "FreeDX".to_string(),
                });
            }
        };

        let base_url = match instrument.market_type {
            MarketType::Spot => "https://api.exchange.freedx.com/spot/api/v3.3/ohlcv",
            MarketType::Derivatives => "https://api.exchange.freedx.com/futures/api/v2.3/ohlcv",
        };

        let limit = instrument.limit.unwrap_or(300).min(300);
        let mut url = format!("{}?symbol={}&resolution={}&limit={}", base_url, instrument.pair, resolution, limit);

        if let Some(start) = instrument.start_time {
            url.push_str(&format!("&start={}", start));
        }
        if let Some(end) = instrument.end_time {
            url.push_str(&format!("&end={}", end));
        }

        let response = reqwest::get(&url)
            .await
            .map_err(|e| CandlesError::ApiError(format!("Failed to fetch candles from FreeDX: {e}")))?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            return Err(CandlesError::HttpError { status, body });
        }

        let candles_api: Vec<Vec<Value>> = response.json().await.map_err(|e| CandlesError::JsonParseError(format!("FreeDX: {e}")))?;

        let mut candles = Vec::with_capacity(candles_api.len());

        // Response is newest-first
        for (index, candle_array) in candles_api.iter().enumerate().rev() {
            if candle_array.len() < 6 {
                return Err(CandlesError::InvalidDataFormat {
                    index,
                    message: format!("Insufficient data in candle array: expected at least 6 elements, got {}", candle_array.len()),
                });
            }

            // Timestamps are in seconds
            let timestamp_secs = candle_array[0].as_i64().ok_or_else(|| CandlesError::ParseError {
                field: "timestamp".to_string(),
                message: format!("at index {} with value {}", index, candle_array[0]),
            })?;

            let close = parse_string_to_f64(&candle_array[4], "close price", index)?;
            // API returns volume in quote asset; convert to base
            let quote_volume = parse_string_to_f64(&candle_array[5], "volume", index)?;
            let volume = if close > 0.0 { quote_volume / close } else { 0.0 };

            candles.push(Candle {
                timestamp: timestamp_secs * 1000,
                open: parse_string_to_f64(&candle_array[1], "open price", index)?,
                high: parse_string_to_f64(&candle_array[2], "high price", index)?,
                low: parse_string_to_f64(&candle_array[3], "low price", index)?,
                close,
                volume,
            });
        }

        Ok(candles)
    }
}
