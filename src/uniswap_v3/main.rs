use alloy::primitives::{Address, B256, I256};
use alloy::providers::{Provider, ProviderBuilder};
use alloy::rpc::types::{Filter, Log};
use async_trait::async_trait;
use std::collections::BTreeMap;
use std::time::Duration;
use tokio::time::sleep;

use crate::{
    base::BaseConnection,
    errors::CandlesError,
    types::{Candle, Instrument, Timeframe},
};

use super::types::{Chain, ProcessedSwap};

pub struct UniswapV3;

const SWAP_EVENT_SIGNATURE: B256 = B256::new([
    0xc4, 0x20, 0x79, 0xf9, 0x4a, 0x63, 0x50, 0xd7, 0xe6, 0x23, 0x5f, 0x29, 0x17, 0x49, 0x24, 0xf9, 0x28, 0xcc, 0x2a, 0xc8, 0x18, 0xeb, 0x64, 0xfe, 0xd8, 0x00, 0x4e, 0x11, 0x5f,
    0xbc, 0xca, 0x67,
]);

impl UniswapV3 {
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

    async fn fetch_swaps_via_rpc(chain: Chain, pool_address: &str, limit: usize) -> Result<Vec<Log>, CandlesError> {
        let rpc_url = std::env::var("RPC_URL").unwrap_or_else(|_| chain.get_rpc_url());

        let provider = ProviderBuilder::new().on_http(rpc_url.parse().map_err(|e| CandlesError::Other(format!("Invalid RPC URL: {e}")))?);

        let current_block = provider
            .get_block_number()
            .await
            .map_err(|e| CandlesError::Other(format!("Failed to get block number: {e}")))?;

        let pool_addr: Address = pool_address.parse().map_err(|e| CandlesError::Other(format!("Invalid pool address: {e}")))?;

        let batch_size = std::env::var("UNISWAP_BATCH_SIZE").ok().and_then(|s| s.parse::<u64>().ok()).unwrap_or(1000);

        let total_blocks_needed = ((limit * 10).max(10000) as u64).min(50000);
        let from_block = current_block.saturating_sub(total_blocks_needed);

        let mut all_logs = Vec::new();
        let mut current_from = from_block;

        while current_from < current_block && all_logs.len() < limit {
            let batch_to = (current_from + batch_size - 1).min(current_block);

            let filter = Filter::new().address(pool_addr).select(current_from..=batch_to).event_signature(vec![SWAP_EVENT_SIGNATURE]);

            let batch_logs = provider
                .get_logs(&filter)
                .await
                .map_err(|e| CandlesError::Other(format!("Failed to fetch logs for blocks {}-{}: {e}", current_from, batch_to)))?;

            all_logs.extend(batch_logs);
            current_from = batch_to + 1;

            if all_logs.len() >= limit {
                break;
            }

            sleep(Duration::from_millis(50)).await;
        }

        Ok(all_logs)
    }

    fn decode_swap_log(log: &Log) -> Result<(i64, I256, I256), CandlesError> {
        if log.topics().len() < 1 || log.data().data.len() < 128 {
            return Err(CandlesError::Other("Invalid log format".to_string()));
        }

        let timestamp = log.block_number.ok_or_else(|| CandlesError::Other("Missing block number".to_string()))? as i64 * 12 * 1000;

        let data = &log.data().data;

        let amount0_bytes: [u8; 32] = data[0..32].try_into().map_err(|_| CandlesError::Other("Failed to parse amount0".to_string()))?;
        let amount0 = I256::from_be_bytes(amount0_bytes);

        let amount1_bytes: [u8; 32] = data[32..64].try_into().map_err(|_| CandlesError::Other("Failed to parse amount1".to_string()))?;
        let amount1 = I256::from_be_bytes(amount1_bytes);

        Ok((timestamp, amount0, amount1))
    }

    fn process_logs(logs: Vec<Log>, invert_price: bool) -> Result<Vec<ProcessedSwap>, CandlesError> {
        let mut processed = Vec::new();

        for log in logs {
            let (timestamp, amount0, amount1) = match Self::decode_swap_log(&log) {
                Ok(data) => data,
                Err(_) => continue,
            };

            let amt0 = if amount0.is_negative() {
                -(amount0.abs().to_string().parse::<f64>().unwrap_or(0.0))
            } else {
                amount0.to_string().parse::<f64>().unwrap_or(0.0)
            };
            let amt1 = if amount1.is_negative() {
                -(amount1.abs().to_string().parse::<f64>().unwrap_or(0.0))
            } else {
                amount1.to_string().parse::<f64>().unwrap_or(0.0)
            };

            if amt0.abs() < f64::EPSILON || amt1.abs() < f64::EPSILON {
                continue;
            }

            let price = if invert_price { amt0.abs() / amt1.abs() } else { amt1.abs() / amt0.abs() };

            let volume_usd = amt0.abs().max(amt1.abs());

            processed.push(ProcessedSwap { timestamp, price, volume_usd });
        }

        Ok(processed)
    }

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
        let logs = Self::fetch_swaps_via_rpc(chain, &pool_address, limit).await?;

        if logs.is_empty() {
            return Err(CandlesError::Other(format!("No swaps found for pool: {pool_address} on chain: {chain}")));
        }

        let processed_swaps = Self::process_logs(logs, invert_price)?;
        let candles = Self::aggregate_to_candles(processed_swaps, &instrument.timeframe)?;

        Ok(candles)
    }
}
