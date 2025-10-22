use async_trait::async_trait;
use serde_json::Value;

use crate::{
    errors::CandlesError,
    providers::base::BaseConnection,
    providers::mexc::types::MexcKlineFuturesResponse,
    types::{Candle, Instrument, MarketType, Timeframe},
    utils::{DataWrapper, parse_string_to_f64},
};

pub struct Mexc;

#[async_trait]
impl BaseConnection for Mexc {
    async fn get_candles(instrument: Instrument) -> Result<Vec<Candle>, crate::errors::CandlesError> {
        match instrument.market_type {
            MarketType::Spot => {
                let mexc_timeframe = match instrument.timeframe {
                    Timeframe::M3 => return Err(CandlesError::Other("m3 Timeframe is not available for Mexc".to_string())),
                    Timeframe::M5 => "5m",
                    Timeframe::M15 => "15m",
                    Timeframe::M30 => "30m",
                    Timeframe::H1 => "60m",
                    Timeframe::H4 => "4h",
                    Timeframe::D1 => "1d",
                    Timeframe::W1 => "1W",
                    Timeframe::MN1 => "1M",
                };

                let url = format!("https://api.mexc.com/api/v3/klines?symbol={}&interval={}", instrument.pair, mexc_timeframe);

                let response: Vec<Vec<Value>> = reqwest::get(&url).await?.json().await?;
                let mut candles = Vec::with_capacity(response.len());

                for (index, candle_array) in response.into_iter().enumerate() {
                    if candle_array.len() < 6 {
                        return Err(CandlesError::Other(format!(
                            "Insufficient data in candle array at index {index}: expected at least 6 elements, got {}",
                            candle_array.len()
                        )));
                    }

                    candles.push(Candle {
                        timestamp: candle_array[0]
                            .as_i64()
                            .ok_or_else(|| CandlesError::Other(format!("Failed to parse timestamp at index {} with value {}", index, candle_array[0])))?,
                        open: parse_string_to_f64(&candle_array[1], "open price", index)?,
                        high: parse_string_to_f64(&candle_array[2], "high price", index)?,
                        low: parse_string_to_f64(&candle_array[3], "low price", index)?,
                        close: parse_string_to_f64(&candle_array[4], "close price", index)?,
                        volume: parse_string_to_f64(&candle_array[5], "volume", index)?,
                    });
                }

                Ok(candles)
            }

            MarketType::Derivatives => {
                let mexc_timeframe = match instrument.timeframe {
                    Timeframe::M3 => return Err(CandlesError::Other("m3 Timeframe is not available for Mexc".to_string())),
                    Timeframe::M5 => "Min5",
                    Timeframe::M15 => "Min15",
                    Timeframe::M30 => "Min30",
                    Timeframe::H1 => "Min60",
                    Timeframe::H4 => "Hour4",
                    Timeframe::D1 => "Day1",
                    Timeframe::W1 => "Week1",
                    Timeframe::MN1 => "Month1",
                };

                let url = format!("https://contract.mexc.com/api/v1/contract/kline/{}?interval={}", instrument.pair, mexc_timeframe);
                let response: DataWrapper<MexcKlineFuturesResponse> = reqwest::get(&url).await?.json().await?;

                let mut candles = Vec::with_capacity(response.data.time.len());

                for (index, timestamp) in response.data.time.into_iter().enumerate() {
                    candles.push(Candle {
                        timestamp: timestamp * 1000,
                        open: response.data.open[index],
                        high: response.data.high[index],
                        low: response.data.low[index],
                        close: response.data.close[index],
                        volume: response.data.vol[index],
                    });
                }

                Ok(candles)
            }
        }
    }
}
