use async_trait::async_trait;
use serde_json::Value;

use crate::{
    errors::CandlesError,
    providers::{
        alpha_vantage::utils::{get_api_key, parse_f64, parse_timestamp},
        base::BaseConnection,
    },
    types::{Candle, Instrument, Timeframe},
};

pub struct AlphaVantage;

#[async_trait]
impl BaseConnection for AlphaVantage {
    async fn get_candles(instrument: Instrument) -> Result<Vec<Candle>, CandlesError> {
        let (function, interval, time_series_key) = match instrument.timeframe {
            Timeframe::M5 => ("TIME_SERIES_INTRADAY", Some("5min"), "Time Series (5min)"),
            Timeframe::M15 => ("TIME_SERIES_INTRADAY", Some("15min"), "Time Series (15min)"),
            Timeframe::M30 => ("TIME_SERIES_INTRADAY", Some("30min"), "Time Series (30min)"),
            Timeframe::H1 => ("TIME_SERIES_INTRADAY", Some("60min"), "Time Series (60min)"),
            Timeframe::D1 => ("TIME_SERIES_DAILY", None, "Time Series (Daily)"),
            Timeframe::W1 => ("TIME_SERIES_WEEKLY", None, "Weekly Time Series"),
            Timeframe::MN1 => ("TIME_SERIES_MONTHLY", None, "Monthly Time Series"),
            _ => {
                return Err(CandlesError::UnsupportedTimeframe {
                    timeframe: format!("{:?}", instrument.timeframe),
                    provider: "AlphaVantage".to_string(),
                });
            }
        };

        let api_key = get_api_key()?;
        let symbol = &instrument.pair;

        let mut url = format!("https://www.alphavantage.co/query?function={function}&symbol={symbol}&apikey={api_key}");

        if let Some(interval) = interval {
            url.push_str(&format!("&interval={interval}&outputsize=full"));
        }

        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| CandlesError::ApiError(format!("Failed to fetch data: {e}")))?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            return Err(CandlesError::HttpError { status, body });
        }

        let json: Value = response.json().await.map_err(|e| CandlesError::JsonParseError(format!("Failed to parse response: {e}")))?;

        if let Some(error_message) = json.get("Error Message") {
            return Err(CandlesError::ApiError(error_message.as_str().unwrap_or("Unknown error").to_string()));
        }

        if let Some(note) = json.get("Note") {
            return Err(CandlesError::ApiError(note.as_str().unwrap_or("API rate limit reached").to_string()));
        }

        let time_series = json
            .get(time_series_key)
            .ok_or_else(|| CandlesError::JsonParseError(format!("Missing '{time_series_key}' in response")))?;

        let time_series_map = time_series
            .as_object()
            .ok_or_else(|| CandlesError::JsonParseError("Time series is not an object".to_string()))?;

        let has_time = interval.is_some();
        let mut candles = Vec::with_capacity(time_series_map.len());

        for (date_str, candle_data) in time_series_map {
            let obj = candle_data.as_object().ok_or_else(|| CandlesError::InvalidDataFormat {
                index: 0,
                message: format!("Candle data at {date_str} is not an object"),
            })?;

            let open = obj.get("1. open").and_then(|v| v.as_str()).ok_or_else(|| CandlesError::MissingField {
                field: "1. open".to_string(),
                index: 0,
            })?;
            let high = obj.get("2. high").and_then(|v| v.as_str()).ok_or_else(|| CandlesError::MissingField {
                field: "2. high".to_string(),
                index: 0,
            })?;
            let low = obj.get("3. low").and_then(|v| v.as_str()).ok_or_else(|| CandlesError::MissingField {
                field: "3. low".to_string(),
                index: 0,
            })?;
            let close = obj.get("4. close").and_then(|v| v.as_str()).ok_or_else(|| CandlesError::MissingField {
                field: "4. close".to_string(),
                index: 0,
            })?;
            let volume = obj.get("5. volume").and_then(|v| v.as_str()).ok_or_else(|| CandlesError::MissingField {
                field: "5. volume".to_string(),
                index: 0,
            })?;

            candles.push(Candle {
                timestamp: parse_timestamp(date_str, has_time)?,
                open: parse_f64(open, "open", date_str)?,
                high: parse_f64(high, "high", date_str)?,
                low: parse_f64(low, "low", date_str)?,
                close: parse_f64(close, "close", date_str)?,
                volume: parse_f64(volume, "volume", date_str)?,
            });
        }

        candles.sort_by_key(|c| c.timestamp);

        Ok(candles)
    }
}
