use async_trait::async_trait;
use serde_json::json;
use std::collections::BTreeMap;

use crate::{
    base::BaseConnection,
    errors::CandlesError,
    types::{Candle, Instrument, Timeframe},
};

use super::types::{Chain, GraphQLResponse, ProcessedSwap, Swap};

pub struct UniswapV3;

impl UniswapV3 {
    /// Parse the instrument pair format: "chain_poolAddress" or "chain_poolAddress_inverted"
    /// Example: "ethereum_0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640"
    /// Example with inversion: "ethereum_0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640_inverted"
    fn parse_pair(pair: &str) -> Result<(Chain, String, bool), CandlesError> {
        let parts: Vec<&str> = pair.split('_').collect();
        if parts.len() < 2 {
            return Err(CandlesError::Other(format!(
                "Invalid pair format. Expected 'chain_poolAddress' or 'chain_poolAddress_inverted', got: {pair}"
            )));
        }

        let chain = Chain::from_str(parts[0]).ok_or_else(|| CandlesError::Other(format!("Unsupported chain: {}", parts[0])))?;

        let pool_address = parts[1].to_lowercase();
        let invert_price = parts.iter().skip(2).any(|&s| s == "inverted");

        Ok((chain, pool_address, invert_price))
    }

    /// Build GraphQL query to fetch swap events
    fn build_swaps_query(pool_address: &str, limit: i32, skip: i32) -> String {
        format!(
            r#"
            {{
                swaps(
                    first: {limit}
                    skip: {skip}
                    orderBy: timestamp
                    orderDirection: desc
                    where: {{ pool: "{pool_address}" }}
                ) {{
                    timestamp
                    amount0
                    amount1
                    amountUSD
                    sqrtPriceX96
                    tick
                }}
            }}
            "#,
        )
    }

    /// Fetch swaps from The Graph subgraph
    async fn fetch_swaps(chain: Chain, pool_address: &str, limit: i32) -> Result<Vec<Swap>, CandlesError> {
        let api_key = std::env::var("GRAPH_API_KEY").expect("GRAPH_API_KEY environment variable must be set");

        let client = reqwest::Client::new();
        let url = chain.get_subgraph_url(&api_key);

        let mut all_swaps = Vec::new();
        let batch_size = 1000;
        let total_batches = ((limit as f64) / (batch_size as f64)).ceil() as i32;

        for batch in 0..total_batches {
            let skip = batch * batch_size;
            let query = Self::build_swaps_query(pool_address, batch_size, skip);

            let response = client.post(&url).json(&json!({ "query": query })).send().await?;

            if !response.status().is_success() {
                return Err(CandlesError::ApiError(format!("GraphQL request failed with status: {}", response.status())));
            }

            let response_text = response.text().await?;

            let graphql_response: GraphQLResponse =
                serde_json::from_str(&response_text).map_err(|e| CandlesError::ApiError(format!("Failed to parse GraphQL response: {e}. Response body: {response_text}",)))?;

            let swaps = graphql_response.data.swaps;
            if swaps.is_empty() {
                break;
            }

            all_swaps.extend(swaps);
        }

        Ok(all_swaps)
    }

    /// Calculate price from swap amounts
    /// If invert=true, returns token0/token1, otherwise token1/token0
    fn calculate_price_from_amounts(amount_0: &str, amount_1: &str, invert: bool) -> Result<f64, CandlesError> {
        let amt0 = amount_0.parse::<f64>().map_err(|e| CandlesError::Other(format!("Failed to parse amount0: {e}")))?;

        let amt1 = amount_1.parse::<f64>().map_err(|e| CandlesError::Other(format!("Failed to parse amount1: {e}")))?;

        if amt0.abs() < f64::EPSILON || amt1.abs() < f64::EPSILON {
            return Err(CandlesError::Other("amount is zero".to_string()));
        }

        let price = if invert { amt0.abs() / amt1.abs() } else { amt1.abs() / amt0.abs() };

        Ok(price)
    }

    /// Process raw swaps into structured data
    fn process_swaps(swaps: Vec<Swap>, invert_price: bool) -> Result<Vec<ProcessedSwap>, CandlesError> {
        let mut processed = Vec::new();

        for swap in swaps {
            let timestamp = swap.timestamp.parse::<i64>().map_err(|e| CandlesError::Other(format!("Failed to parse timestamp: {e}")))? * 1000;

            let price = match Self::calculate_price_from_amounts(&swap.amount_0, &swap.amount_1, invert_price) {
                Ok(p) => p,
                Err(_) => continue,
            };

            let volume_usd = swap
                .amount_usd
                .parse::<f64>()
                .map_err(|e| CandlesError::Other(format!("Failed to parse amountUSD: {e}")))?
                .abs();

            processed.push(ProcessedSwap { timestamp, price, volume_usd });
        }

        Ok(processed)
    }

    /// Get timeframe duration in milliseconds
    fn get_timeframe_ms(timeframe: &Timeframe) -> i64 {
        match timeframe {
            Timeframe::M3 => 3 * 60 * 1000,
            Timeframe::M5 => 5 * 60 * 1000,
            Timeframe::M15 => 15 * 60 * 1000,
            Timeframe::M30 => 30 * 60 * 1000,
            Timeframe::H1 => 60 * 60 * 1000,
            Timeframe::H4 => 4 * 60 * 60 * 1000,
            Timeframe::D1 => 24 * 60 * 60 * 1000,
            Timeframe::W1 => 7 * 24 * 60 * 60 * 1000,
            Timeframe::MN1 => 30 * 24 * 60 * 60 * 1000,
        }
    }

    /// Aggregate swaps into OHLCV candles
    fn aggregate_to_candles(swaps: Vec<ProcessedSwap>, timeframe: &Timeframe) -> Result<Vec<Candle>, CandlesError> {
        if swaps.is_empty() {
            return Ok(Vec::new());
        }

        let timeframe_ms = Self::get_timeframe_ms(timeframe);
        let mut candle_map: BTreeMap<i64, Vec<ProcessedSwap>> = BTreeMap::new();

        for swap in swaps {
            let candle_time = (swap.timestamp / timeframe_ms) * timeframe_ms;
            candle_map.entry(candle_time).or_default().push(swap);
        }

        let mut candles = Vec::with_capacity(candle_map.len());

        for (timestamp, mut period_swaps) in candle_map {
            if period_swaps.is_empty() {
                continue;
            }

            period_swaps.sort_by_key(|s| s.timestamp);

            let open = period_swaps.first().unwrap().price;
            let close = period_swaps.last().unwrap().price;

            let high = period_swaps.iter().map(|s| s.price).fold(f64::NEG_INFINITY, f64::max);

            let low = period_swaps.iter().map(|s| s.price).fold(f64::INFINITY, f64::min);

            let volume = period_swaps.iter().map(|s| s.volume_usd).sum();

            candles.push(Candle {
                timestamp,
                open,
                high,
                low,
                close,
                volume,
            });
        }

        Ok(candles)
    }
}

#[async_trait]
impl BaseConnection for UniswapV3 {
    async fn get_candles(instrument: Instrument) -> Result<Vec<Candle>, CandlesError> {
        let (chain, pool_address, invert_price) = Self::parse_pair(&instrument.pair)?;

        let limit = 30000;
        let swaps = Self::fetch_swaps(chain, &pool_address, limit).await?;

        if swaps.is_empty() {
            return Err(CandlesError::Other(format!("No swaps found for pool: {pool_address} on chain: {chain}")));
        }

        let processed_swaps = Self::process_swaps(swaps, invert_price)?;
        let candles = Self::aggregate_to_candles(processed_swaps, &instrument.timeframe)?;

        Ok(candles)
    }
}
