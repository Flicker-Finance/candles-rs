use async_trait::async_trait;
use serde_json::Value;

use crate::{
    errors::CandlesError,
    providers::base::BaseConnection,
    types::{Candle, Instrument, Timeframe},
    utils::{DataWrapper, parse_string_to_f64},
};

pub struct OKX;

#[async_trait]
impl BaseConnection for OKX {
    async fn get_candles(instrument: crate::types::Instrument) -> Result<Vec<crate::types::Candle>, crate::errors::CandlesError> {
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

        let limit = instrument.limit.unwrap_or(100).min(100);

        // /candles only stores the latest 1,440 entries; /history-candles goes back years
        let endpoint = if let Some(end_time) = instrument.end_time {
            let now_ms = chrono::Utc::now().timestamp_millis();
            let history_border = now_ms - 1440 * instrument.timeframe.to_ms();
            if end_time < history_border { "history-candles" } else { "candles" }
        } else {
            "candles"
        };

        let mut candles = Self::fetch_okx_candles(&instrument, okx_timeframe, limit, endpoint).await?;

        if endpoint == "candles" && (candles.len() as u64) < limit && !candles.is_empty() {
            if let Some(oldest) = candles.first().map(|c| c.timestamp) {
                let remaining = limit - candles.len() as u64;
                let mut hist_instrument = instrument.clone();
                hist_instrument.end_time = Some(oldest);
                let history_candles = Self::fetch_okx_candles(&hist_instrument, okx_timeframe, remaining, "history-candles").await?;
                if !history_candles.is_empty() {
                    let mut combined = history_candles;
                    combined.append(&mut candles);
                    candles = combined;
                }
            }
        }

        Ok(candles)
    }
}

impl OKX {
    async fn fetch_okx_candles(instrument: &Instrument, okx_timeframe: &str, limit: u64, endpoint: &str) -> Result<Vec<Candle>, CandlesError> {
        let mut url = format!(
            "https://www.okx.com/api/v5/market/{}?instId={}&bar={}&limit={}",
            endpoint, instrument.pair, okx_timeframe, limit
        );

        if let Some(end_time) = instrument.end_time {
            url.push_str(&format!("&after={}", end_time));
        }
        if let Some(start_time) = instrument.start_time {
            url.push_str(&format!("&before={}", start_time));
        }

        let response = reqwest::get(&url)
            .await
            .map_err(|e| CandlesError::ApiError(format!("Failed to fetch candles from OKX: {e}")))?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            return Err(CandlesError::HttpError { status, body });
        }

        let response_body: DataWrapper<Vec<Value>> = response.json().await.map_err(|e| CandlesError::JsonParseError(format!("OKX: {e}")))?;

        let mut candles = Vec::with_capacity(response_body.data.len());

        for (index, value) in response_body.data.iter().enumerate().rev() {
            let candle_array = value.as_array().ok_or_else(|| CandlesError::InvalidDataFormat {
                index,
                message: "Expected array for candle data".to_string(),
            })?;

            if candle_array.len() < 6 {
                return Err(CandlesError::InvalidDataFormat {
                    index,
                    message: format!("Insufficient data in candle array: expected at least 6 elements, got {}", candle_array.len()),
                });
            }

            candles.push(Candle {
                timestamp: candle_array[0]
                    .as_str()
                    .ok_or_else(|| CandlesError::ParseError {
                        field: "timestamp".to_string(),
                        message: format!("at index {} with value {}", index, candle_array[0]),
                    })?
                    .parse::<i64>()
                    .map_err(|_| CandlesError::ParseError {
                        field: "timestamp".to_string(),
                        message: format!("at index {} with value {}", index, candle_array[0]),
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
