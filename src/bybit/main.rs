use async_trait::async_trait;

use crate::{
    base::BaseConnection,
    bybit::types::BybitKlineResponse,
    errors::CandlesError,
    types::{Candle, MarketType, Timeframe},
    utils::{ResultWrapper, parse_string_to_f64},
};

pub struct Bybit;

#[async_trait]
impl BaseConnection for Bybit {
    async fn get_candles(
        instrument: crate::types::Instrument,
    ) -> Result<Vec<crate::types::Candle>, crate::errors::CandlesError> {
        let bybit_timeframe = match instrument.timeframe {
            Timeframe::M3 => "3",
            Timeframe::M5 => "5",
            Timeframe::M15 => "15",
            Timeframe::M30 => "30",
            Timeframe::H1 => "60",
            Timeframe::H4 => "240",
            Timeframe::D1 => "D",
            Timeframe::W1 => "W",
            Timeframe::MN1 => "M",
        };

        let category = match instrument.market_type {
            MarketType::Spot => "spot",
            MarketType::Derivatives => "linear",
        };

        let url = format!(
            "https://api.bybit.com/v5/market/kline?category={}&symbol={}&interval={}",
            category, instrument.pair, bybit_timeframe
        );

        let response: ResultWrapper<BybitKlineResponse> = reqwest::get(&url).await?.json().await?;

        let mut candles = Vec::with_capacity(response.result.list.len());

        for (index, value) in response.result.list.iter().enumerate().rev() {
            let candle_array = value.as_array().ok_or_else(|| {
                CandlesError::Other(format!("Expected array for candle data at index {index}"))
            })?;

            if candle_array.len() < 6 {
                return Err(CandlesError::Other(format!(
                    "Insufficient data in candle array at index {index}: expected at least 6 elements, got {}",
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
