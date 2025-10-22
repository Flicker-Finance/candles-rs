use async_trait::async_trait;
use serde_json::Value;

use crate::{
    errors::CandlesError,
    providers::base::BaseConnection,
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
                        timestamp: candle_array[0].as_i64().ok_or(CandlesError::ParseError {
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
                MarketType::Derivatives => {
                    let candle_object = value.as_object().ok_or_else(|| CandlesError::InvalidDataFormat {
                        index,
                        message: "Expected object for candle data".to_string(),
                    })?;

                    if candle_object.keys().len() < 6 {
                        return Err(CandlesError::InvalidDataFormat {
                            index,
                            message: format!("Insufficient data in candle object: expected at least 6 elements, got {}", candle_object.len()),
                        });
                    }

                    candles.push(Candle {
                        timestamp: candle_object
                            .get("time")
                            .ok_or(CandlesError::MissingField { field: "time".to_string(), index })?
                            .as_i64()
                            .ok_or(CandlesError::ParseError {
                                field: "timestamp".to_string(),
                                message: format!("at index {index}"),
                            })?,
                        open: parse_string_to_f64(
                            candle_object.get("open").ok_or(CandlesError::MissingField { field: "open".to_string(), index })?,
                            "open price",
                            index,
                        )?,
                        high: parse_string_to_f64(
                            candle_object.get("high").ok_or(CandlesError::MissingField { field: "high".to_string(), index })?,
                            "high price",
                            index,
                        )?,
                        low: parse_string_to_f64(
                            candle_object.get("low").ok_or(CandlesError::MissingField { field: "low".to_string(), index })?,
                            "low price",
                            index,
                        )?,
                        close: parse_string_to_f64(
                            candle_object.get("close").ok_or(CandlesError::MissingField {
                                field: "close".to_string(),
                                index,
                            })?,
                            "close price",
                            index,
                        )?,
                        volume: parse_string_to_f64(
                            candle_object.get("volume").ok_or(CandlesError::MissingField {
                                field: "volume".to_string(),
                                index,
                            })?,
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
