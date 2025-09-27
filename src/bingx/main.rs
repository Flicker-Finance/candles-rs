use async_trait::async_trait;
use serde_json::Value;

use crate::{
    base::BaseConnection,
    errors::CandlesError,
    types::{Candle, Instrument, MarketType, Timeframe},
    utils::{DataWrapper, parse_string_to_f64},
};

pub struct BingX;

#[async_trait]
impl BaseConnection for BingX {
    async fn get_candles(instrument: Instrument) -> Result<Vec<Candle>, CandlesError> {
        let bingx_timeframe = match instrument.timeframe {
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

        let path = match instrument.market_type {
            MarketType::Spot => "/openApi/spot/v2/market/kline",
            MarketType::Derivatives => "/openApi/swap/v3/quote/klines",
        };

        let url = format!("https://open-api.bingx.com{path}?symbol={}&interval={}", instrument.pair, bingx_timeframe);

        let response: DataWrapper<Vec<Value>> = reqwest::get(&url).await?.json().await?;

        let mut candles = Vec::with_capacity(response.data.len());

        for (index, value) in response.data.iter().enumerate().rev() {
            match instrument.market_type {
                MarketType::Spot => {
                    let candle_array = value
                        .as_array()
                        .ok_or_else(|| CandlesError::Other(format!("Expected array for candle data at index {index}")))?;

                    if candle_array.len() < 6 {
                        return Err(CandlesError::Other(format!(
                            "Insufficient data in candle array at index {index}: expected at least 6 elements, got {}",
                            candle_array.len()
                        )));
                    }

                    candles.push(Candle {
                        timestamp: candle_array[6]
                            .as_i64()
                            .ok_or(CandlesError::Other(format!("Failed to parse timestamp at index {} with value {}", index, candle_array[0])))?,
                        open: parse_string_to_f64(&candle_array[1], "open price", index)?,
                        high: parse_string_to_f64(&candle_array[2], "high price", index)?,
                        low: parse_string_to_f64(&candle_array[3], "low price", index)?,
                        close: parse_string_to_f64(&candle_array[4], "close price", index)?,
                        volume: parse_string_to_f64(&candle_array[5], "volume", index)?,
                    });
                }
                MarketType::Derivatives => {
                    let candle_object = value
                        .as_object()
                        .ok_or_else(|| CandlesError::Other(format!("Expected object for candle data at index {index}")))?;

                    if candle_object.keys().len() < 6 {
                        return Err(CandlesError::Other(format!(
                            "Insufficient data in candle object at index {index}: expected at least 6 elements, got {}",
                            candle_object.len()
                        )));
                    }

                    candles.push(Candle {
                        timestamp: candle_object
                            .get("time")
                            .ok_or(CandlesError::Other(format!("Failed to get timestamp for key 'time' at index {index}")))?
                            .as_i64()
                            .ok_or(CandlesError::Other(format!("Failed to parse timestamp at index {index}")))?,
                        open: parse_string_to_f64(
                            candle_object.get("open").ok_or(CandlesError::Other(format!("Failed to get 'open' at index {index}")))?,
                            "open price",
                            index,
                        )?,
                        high: parse_string_to_f64(
                            candle_object.get("high").ok_or(CandlesError::Other(format!("Failed to get 'high' at index {index}")))?,
                            "high price",
                            index,
                        )?,
                        low: parse_string_to_f64(
                            candle_object.get("low").ok_or(CandlesError::Other(format!("Failed to get 'low' at index {index}")))?,
                            "low price",
                            index,
                        )?,
                        close: parse_string_to_f64(
                            candle_object.get("close").ok_or(CandlesError::Other(format!("Failed to get 'close' at index {index}")))?,
                            "close price",
                            index,
                        )?,
                        volume: parse_string_to_f64(
                            candle_object.get("volume").ok_or(CandlesError::Other(format!("Failed to get 'volume' at index {index}")))?,
                            "volume",
                            index,
                        )?,
                    });
                }
            }
        }

        Ok(candles)
    }
}
