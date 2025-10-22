use async_trait::async_trait;
use serde_json::Value;

use crate::{
    errors::CandlesError,
    providers::base::BaseConnection,
    types::{Candle, Instrument, MarketType, Timeframe},
    utils::parse_string_to_f64,
};

pub struct Binance;

#[async_trait]
impl BaseConnection for Binance {
    async fn get_candles(instrument: Instrument) -> Result<Vec<Candle>, CandlesError> {
        let binance_timeframe = match instrument.timeframe {
            Timeframe::M3 => "3m",
            Timeframe::M5 => "5m",
            Timeframe::M15 => "15m",
            Timeframe::M30 => "30m",
            Timeframe::H1 => "1h",
            Timeframe::H4 => "4h",
            Timeframe::D1 => "1d",
            Timeframe::W1 => "1w",
            Timeframe::MN1 => "1M",
        };

        let url = match instrument.market_type {
            MarketType::Spot => format!("https://www.binance.com/api/v3/klines?symbol={}&interval={}", instrument.pair, binance_timeframe),
            MarketType::Derivatives => format!("https://fapi.binance.com/fapi/v1/klines?symbol={}&interval={}", instrument.pair, binance_timeframe),
        };

        let response = reqwest::get(&url).await.map_err(|e| CandlesError::ApiError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            return Err(CandlesError::HttpError { status, body });
        }

        let candles_api = response.json::<Vec<Value>>().await.map_err(|e| CandlesError::JsonParseError(e.to_string()))?;

        let mut candles = Vec::with_capacity(candles_api.len());

        for (index, value) in candles_api.iter().enumerate() {
            let candle_array = value.as_array().ok_or(CandlesError::InvalidDataFormat {
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
                timestamp: candle_array[0].as_i64().ok_or(CandlesError::ParseError {
                    field: "timestamp".to_string(),
                    message: format!("at index {index}"),
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
