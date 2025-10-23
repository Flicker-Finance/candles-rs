use async_trait::async_trait;

use crate::{
    errors::CandlesError,
    providers::base::BaseConnection,
    providers::coingecko::types::OhlcvResponse,
    types::{Candle, Instrument, Timeframe},
    utils::parse_string_to_f64,
};

pub struct CoinGecko;

#[async_trait]
impl BaseConnection for CoinGecko {
    async fn get_candles(instrument: Instrument) -> Result<Vec<Candle>, CandlesError> {
        let parts: Vec<&str> = instrument.pair.split('_').collect();
        if parts.len() != 2 {
            return Err(CandlesError::InvalidPairFormat(format!(
                "Expected format: <network>_<pool_address>, got: {}",
                instrument.pair
            )));
        }

        let network = parts[0];
        let pool_address = parts[1];

        let timeframe = match instrument.timeframe.clone() {
            Timeframe::M3 => "minute",
            Timeframe::M5 => "minute",
            Timeframe::M15 => "minute",
            Timeframe::M30 => "minute",
            Timeframe::H1 => "hour",
            Timeframe::H4 => "hour",
            Timeframe::D1 => "day",
            Timeframe::W1 => "day",
            Timeframe::MN1 => "day",
        };
        let url = format!("https://api.geckoterminal.com/api/v2/networks/{network}/pools/{pool_address}/ohlcv/{timeframe}");

        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .header("Accept", "application/json")
            .query(&[("aggregate", "1"), ("limit", "1000")])
            .send()
            .await
            .map_err(|e| CandlesError::ApiError(format!("Failed to fetch OHLCV data: {e}")))?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            return Err(CandlesError::HttpError { status, body });
        }

        let ohlcv_response: OhlcvResponse = response
            .json()
            .await
            .map_err(|e| CandlesError::JsonParseError(format!("Failed to parse GeckoTerminal OHLCV response: {e}")))?;

        let mut candles = Vec::with_capacity(ohlcv_response.data.attributes.ohlcv_list.len());

        for (index, ohlcv_array) in ohlcv_response.data.attributes.ohlcv_list.iter().enumerate().rev() {
            if ohlcv_array.len() < 6 {
                return Err(CandlesError::InvalidDataFormat {
                    index,
                    message: format!("Insufficient data in OHLCV array: expected at least 6 elements, got {}", ohlcv_array.len()),
                });
            }

            candles.push(Candle {
                timestamp: ohlcv_array[0].as_i64().ok_or(CandlesError::ParseError {
                    field: "timestamp".to_string(),
                    message: format!("at index {index}"),
                })? * 1000,
                open: parse_string_to_f64(&ohlcv_array[1], "open price", index)?,
                high: parse_string_to_f64(&ohlcv_array[2], "high price", index)?,
                low: parse_string_to_f64(&ohlcv_array[3], "low price", index)?,
                close: parse_string_to_f64(&ohlcv_array[4], "close price", index)?,
                volume: parse_string_to_f64(&ohlcv_array[5], "volume", index)?,
            });
        }

        Ok(candles)
    }
}
