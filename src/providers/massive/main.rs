use async_trait::async_trait;
use serde::Deserialize;

use crate::{
    errors::CandlesError,
    providers::{
        base::BaseConnection,
        massive::utils::{get_api_key, timeframe_to_massive},
    },
    types::{Candle, Instrument},
};

const BASE_URL: &str = "https://api.massive.com";
const DEFAULT_LIMIT: u64 = 5_000;
const MAX_LIMIT: u64 = 50_000;

/// Earliest `from` value for Massive aggregates when the caller didn't bound
/// `start_time`. Roughly Jan 1, 2000 in milliseconds — well before any
/// US-listed equity has intraday data.
const EARLIEST_FROM_MS: i64 = 946_684_800_000;

pub struct Massive;

#[derive(Debug, Deserialize)]
struct AggregatesResponse {
    #[serde(default)]
    results: Option<Vec<AggregateBar>>,
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    status: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AggregateBar {
    /// Unix millisecond timestamp at the start of the bar.
    t: i64,
    /// Open.
    o: f64,
    /// High.
    h: f64,
    /// Low.
    l: f64,
    /// Close.
    c: f64,
    /// Volume in shares.
    v: f64,
}

#[async_trait]
impl BaseConnection for Massive {
    async fn get_candles(instrument: Instrument) -> Result<Vec<Candle>, CandlesError> {
        let (multiplier, timespan) = timeframe_to_massive(&instrument.timeframe)?;

        // Massive's URL takes ticker as a path segment. We prefer `asset_symbol`
        // ("AAPL") over `pair` ("AAPL-USD") since stocks have no quote currency
        // suffix in the API.
        let ticker = if !instrument.asset_symbol.is_empty() {
            instrument.asset_symbol.as_str()
        } else {
            instrument.pair.as_str()
        };

        let from_ms = instrument.start_time.unwrap_or(EARLIEST_FROM_MS);
        let to_ms = instrument.end_time.unwrap_or_else(|| chrono::Utc::now().timestamp_millis());

        let limit = instrument.limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT);
        let api_key = get_api_key()?;

        // `sort=desc` paired with `limit=N` returns the *most recent* N bars in
        // the [from, to] window — that's what the shared pagination driver in
        // `connections.rs` expects (it then walks backwards via `end_time`).
        let url = format!("{BASE_URL}/v2/aggs/ticker/{ticker}/range/{multiplier}/{timespan}/{from_ms}/{to_ms}?adjusted=true&sort=desc&limit={limit}");

        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .bearer_auth(&api_key)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| CandlesError::ApiError(format!("Failed to fetch data: {e}")))?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            return Err(CandlesError::HttpError { status, body });
        }

        let payload: AggregatesResponse = response.json().await.map_err(|e| CandlesError::JsonParseError(format!("Failed to parse response: {e}")))?;

        if let Some(err) = payload.error {
            return Err(CandlesError::ApiError(err));
        }

        // `status: "ERROR"` is also a thing on Massive; treat anything non-OK
        // with empty results as an API error.
        let bars = payload.results.unwrap_or_default();
        if bars.is_empty()
            && payload
                .status
                .as_deref()
                .is_some_and(|s| !s.eq_ignore_ascii_case("OK") && !s.eq_ignore_ascii_case("DELAYED"))
        {
            return Err(CandlesError::ApiError(format!(
                "Massive returned status='{}' with no results for {ticker}",
                payload.status.unwrap_or_default()
            )));
        }

        let mut candles: Vec<Candle> = bars
            .into_iter()
            .map(|b| Candle {
                timestamp: b.t,
                open: b.o,
                high: b.h,
                low: b.l,
                close: b.c,
                volume: b.v,
            })
            .collect();

        // Driver expects ascending order.
        candles.sort_by_key(|c| c.timestamp);

        Ok(candles)
    }
}
