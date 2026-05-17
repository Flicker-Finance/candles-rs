use std::sync::OnceLock;

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

/// Note on rate limits: Massive's free tier is ~5 req/min and intraday data
/// is 15-minute delayed. A single `get_candles` call here can fire dozens of
/// cursor requests in succession, so this provider is **not usable on the
/// free tier** — it will hit 429s almost immediately. Paid plans only.
fn http_client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(reqwest::Client::new)
}

pub struct Massive;

#[derive(Debug, Deserialize)]
struct AggregatesResponse {
    #[serde(default)]
    results: Option<Vec<AggregateBar>>,
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    next_url: Option<String>,
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

        // `target_total` is BOTH the per-page hint we send to Massive (the
        // `?limit=` query param, which the cursor URL bakes in for subsequent
        // pages) and the total number of candles we collect before breaking
        // out of the cursor loop. They happen to be equal — if you ever want
        // to decouple them, split this into two named values.
        let target_total = instrument.limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT);
        let api_key = get_api_key()?;

        // `sort=desc` paired with `limit=N` returns the *most recent* N bars
        // in the [from, to] window. We then follow `next_url` to walk older.
        let initial_url = format!("{BASE_URL}/v2/aggs/ticker/{ticker}/range/{multiplier}/{timespan}/{from_ms}/{to_ms}?adjusted=true&sort=desc&limit={target_total}");

        let client = http_client();
        let mut all_bars: Vec<AggregateBar> = Vec::new();
        let mut next_url: Option<String> = Some(initial_url);
        let mut first_page = true;

        // Massive often returns a tiny first page (e.g. just the current
        // session's bars) plus a `next_url` cursor for older data. Follow the
        // cursor until we hit `target_total` or run out.
        const MAX_PAGES: u32 = 100;
        let mut pages = 0u32;

        while let Some(url) = next_url.take() {
            if pages >= MAX_PAGES {
                break;
            }
            pages += 1;

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

            let bars = payload.results.unwrap_or_default();

            // Only fail hard on the *first* page — later empty pages just
            // mean we've reached the end of the cursor.
            if first_page
                && bars.is_empty()
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
            first_page = false;

            let got_bars = !bars.is_empty();
            all_bars.extend(bars);

            if all_bars.len() as u64 >= target_total {
                break;
            }

            // Stop if there's no cursor, or if this page was empty (avoid
            // tight-looping on a stuck cursor).
            match payload.next_url {
                Some(next) if got_bars => next_url = Some(next),
                _ => break,
            }
        }

        let mut candles: Vec<Candle> = all_bars
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

        // Caller expects ascending order.
        candles.sort_by_key(|c| c.timestamp);

        // The cursor's page size is opaque to us, so the last page can land
        // us a bit over `target_total` — drop the oldest excess to keep the
        // newest `target_total` bars.
        if candles.len() as u64 > target_total {
            let skip = candles.len() - target_total as usize;
            candles.drain(0..skip);
        }

        Ok(candles)
    }
}
