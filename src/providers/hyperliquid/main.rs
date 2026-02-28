use async_trait::async_trait;
use chrono::Utc;

use crate::{
    errors::CandlesError,
    providers::{
        base::BaseConnection,
        hyperliquid::types::{CandleSnapshotParams, CandleSnapshotRequest, HyperliquidCandle},
    },
    types::{Candle, Instrument, Timeframe},
    utils::parse_string_to_f64,
};

const HYPERLIQUID_API_URL: &str = "https://api.hyperliquid.xyz/info";

pub struct Hyperliquid;

fn get_interval(timeframe: &Timeframe) -> Result<&'static str, CandlesError> {
    match timeframe {
        Timeframe::M3 => Ok("3m"),
        Timeframe::M5 => Ok("5m"),
        Timeframe::M15 => Ok("15m"),
        Timeframe::M30 => Ok("30m"),
        Timeframe::H1 => Ok("1h"),
        Timeframe::H4 => Ok("4h"),
        Timeframe::D1 => Ok("1d"),
        Timeframe::W1 => Ok("1w"),
        Timeframe::MN1 => Ok("1M"),
    }
}

#[async_trait]
impl BaseConnection for Hyperliquid {
    async fn get_candles(instrument: Instrument) -> Result<Vec<Candle>, CandlesError> {
        let interval = get_interval(&instrument.timeframe)?;
        let limit = (instrument.limit.unwrap_or(500).min(5000)) as i64;
        let interval_ms = instrument.timeframe.to_ms();

        let end_time = instrument.end_time.unwrap_or_else(|| Utc::now().timestamp_millis());
        let start_time = instrument.start_time.unwrap_or_else(|| end_time - (limit * interval_ms));

        let request = CandleSnapshotRequest {
            request_type: "candleSnapshot".to_string(),
            req: CandleSnapshotParams {
                coin: instrument.asset_symbol.clone(),
                interval: interval.to_string(),
                start_time,
                end_time,
            },
        };

        let client = reqwest::Client::new();
        let response = client
            .post(HYPERLIQUID_API_URL)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| CandlesError::ApiError(format!("Failed to fetch candles: {e}")))?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            return Err(CandlesError::HttpError { status, body });
        }

        let candles_api: Vec<HyperliquidCandle> = response.json().await.map_err(|e| CandlesError::JsonParseError(format!("Failed to parse response: {e}")))?;

        let mut candles = Vec::with_capacity(candles_api.len());

        for (index, hl_candle) in candles_api.iter().enumerate() {
            candles.push(Candle {
                timestamp: hl_candle.t,
                open: parse_string_to_f64(&serde_json::Value::String(hl_candle.o.clone()), "open", index)?,
                high: parse_string_to_f64(&serde_json::Value::String(hl_candle.h.clone()), "high", index)?,
                low: parse_string_to_f64(&serde_json::Value::String(hl_candle.l.clone()), "low", index)?,
                close: parse_string_to_f64(&serde_json::Value::String(hl_candle.c.clone()), "close", index)?,
                volume: parse_string_to_f64(&serde_json::Value::String(hl_candle.v.clone()), "volume", index)?,
            });
        }

        Ok(candles)
    }
}
