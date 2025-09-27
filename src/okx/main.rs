use async_trait::async_trait;
use serde_json::Value;

use crate::{
    base::BaseConnection,
    errors::CandlesError,
    types::{Candle, Timeframe},
    utils::{DataWrapper, parse_string_to_f64},
};

pub struct OKX;

#[async_trait]
impl BaseConnection for OKX {
    async fn get_candles(
        instrument: crate::types::Instrument,
    ) -> Result<Vec<crate::types::Candle>, crate::errors::CandlesError> {
        let okx_timeframe = match instrument.timeframe {
            Timeframe::M3 => "3m",
            Timeframe::M5 => "5m",
            Timeframe::M15 => "15m",
            Timeframe::M30 => "30m",
            Timeframe::H1 => "1H",
            Timeframe::H4 => "4H",
            Timeframe::D1 => "1D",
            Timeframe::W1 => "1W",
            Timeframe::MN1 => "1M",
        };

        let url = format!(
            "https://www.okx.com/api/v5/market/candles?instId={}&bar={}&limit=300",
            instrument.pair, okx_timeframe
        );

        let response = reqwest::get(&url)
            .await
            .map_err(|e| CandlesError::Other(format!("Failed to fetch candles from OKX: {}", e)))?;

        if !response.status().is_success() {
            return Err(CandlesError::Other(format!(
                "OKX API request failed with status {}: {}",
                response.status(),
                response.text().await.unwrap_or_default()
            )));
        }

        let response_body: DataWrapper<Vec<Value>> = response.json().await.map_err(|e| {
            CandlesError::Other(format!("Failed to parse OKX JSON response: {}", e))
        })?;

        let mut candles = Vec::with_capacity(response_body.data.len());

        for (index, value) in response_body.data.iter().enumerate().rev() {
            let candle_array = value.as_array().ok_or_else(|| {
                CandlesError::Other(format!("Expected array for candle data at index {}", index))
            })?;

            if candle_array.len() < 6 {
                return Err(CandlesError::Other(format!(
                    "Insufficient data in candle array at index {}: expected at least 6 elements, got {}",
                    index,
                    candle_array.len()
                )));
            }

            candles.push(Candle {
                timestamp: candle_array[0]
                    .as_str()
                    .ok_or_else(|| {
                        CandlesError::Other(format!(
                            "Invalid timestamp at index {} with value {}",
                            index, candle_array[0]
                        ))
                    })?
                    .parse::<i64>()
                    .map_err(|_| {
                        CandlesError::Other(format!(
                            "Failed to parse timestamp at index {} with value {}",
                            index, candle_array[0]
                        ))
                    })?,
                open: parse_string_to_f64(&candle_array[1], "open price", index)?,
                high: parse_string_to_f64(&candle_array[2], "high price", index)?,
                low: parse_string_to_f64(&candle_array[3], "low price", index)?,
                close: parse_string_to_f64(&candle_array[4], "close price", index)?,
                volume: parse_string_to_f64(&candle_array[5], "volume", index)?,
            });
        }

        Ok(candles)
    }
}
