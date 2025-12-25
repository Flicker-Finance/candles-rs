use alloy::primitives::{Address, B256, I256, U256, keccak256};
use alloy::providers::{Provider, ProviderBuilder};
use alloy::rpc::types::{Filter, Log};
use alloy::transports::http::Http;
use async_trait::async_trait;
use std::collections::BTreeMap;
use std::time::Duration;
use tokio::time::sleep;

use crate::modules::chains::Chain;
use crate::modules::pairs::Pool;
use crate::{
    errors::CandlesError,
    providers::base::BaseConnection,
    types::{Candle, Instrument, Timeframe},
};

use super::types::ProcessedSwap;

pub struct UniswapV3;

fn swap_event_signature() -> B256 {
    keccak256("Swap(address,uint256,uint256,uint256,uint256,address)")
}

impl UniswapV3 {
    async fn fetch_and_aggregate_swaps(chain: Chain, pool_address: &str, timeframe: &Timeframe, invert_price: bool, limit: usize) -> Result<Vec<Candle>, CandlesError> {
        let rpc_url = std::env::var("RPC_URL").unwrap_or_else(|_| chain.get_rpc_url());

        let provider = ProviderBuilder::new().on_http(rpc_url.parse().map_err(|e| CandlesError::RpcError(format!("Invalid RPC URL: {e}")))?);

        let current_block = provider
            .get_block_number()
            .await
            .map_err(|e| CandlesError::RpcError(format!("Failed to get block number: {e}")))?;

        let pool_addr: Address = pool_address.parse().map_err(|_| CandlesError::InvalidAddress(pool_address.to_string()))?;

        // Fetch token addresses and decimals
        let (token0, token1) = Self::get_pool_tokens(&provider, pool_addr).await?;
        let decimals0 = Self::get_token_decimals(&provider, token0).await?;
        let decimals1 = Self::get_token_decimals(&provider, token1).await?;

        let batch_size = std::env::var("UNISWAP_BATCH_SIZE").ok().and_then(|s| s.parse::<u64>().ok()).unwrap_or(1000);

        let rpc_delay_ms = std::env::var("UNISWAP_RPC_DELAY_MS").ok().and_then(|s| s.parse::<u64>().ok()).unwrap_or(50);

        let max_blocks_to_scan = 200_000;
        let from_block = current_block.saturating_sub(max_blocks_to_scan);

        let timeframe_ms = Self::get_timeframe_ms(timeframe);
        let mut candle_map: BTreeMap<i64, Vec<ProcessedSwap>> = BTreeMap::new();
        let mut current_from = from_block;

        while current_from < current_block {
            let batch_to = (current_from + batch_size - 1).min(current_block);

            let filter = Filter::new()
                .address(pool_addr)
                .select(current_from..=batch_to)
                .event_signature(vec![swap_event_signature()]);

            let batch_logs = provider
                .get_logs(&filter)
                .await
                .map_err(|e| CandlesError::RpcError(format!("Failed to fetch logs for blocks {current_from}-{batch_to}: {e}")))?;

            for log in batch_logs {
                let (timestamp, amount0, amount1) = match Self::decode_swap_log(&log) {
                    Ok(data) => data,
                    Err(_) => continue,
                };

                // Parse raw amounts to f64
                let amt0_raw = if amount0.is_negative() {
                    -(amount0.abs().to_string().parse::<f64>().unwrap_or(0.0))
                } else {
                    amount0.to_string().parse::<f64>().unwrap_or(0.0)
                };
                let amt1_raw = if amount1.is_negative() {
                    -(amount1.abs().to_string().parse::<f64>().unwrap_or(0.0))
                } else {
                    amount1.to_string().parse::<f64>().unwrap_or(0.0)
                };

                // Normalize by token decimals
                let amt0 = amt0_raw / 10_f64.powi(decimals0 as i32);
                let amt1 = amt1_raw / 10_f64.powi(decimals1 as i32);

                if amt0.abs() < f64::EPSILON || amt1.abs() < f64::EPSILON {
                    continue;
                }

                let price = if invert_price { amt0.abs() / amt1.abs() } else { amt1.abs() / amt0.abs() };
                let volume_usd = amt0.abs().max(amt1.abs());

                let candle_time = (timestamp / timeframe_ms) * timeframe_ms;
                candle_map.entry(candle_time).or_default().push(ProcessedSwap { timestamp, price, volume_usd });
            }

            current_from = batch_to + 1;

            if candle_map.len() >= limit {
                break;
            }

            sleep(Duration::from_millis(rpc_delay_ms)).await;
        }

        if candle_map.is_empty() {
            return Err(CandlesError::InvalidBlockchainData(format!(
                "No swaps found for pool: {pool_address} on chain: {chain}. \
                Make sure this is a Uniswap V3 pool address (not a router). \
                Find pool addresses at https://info.uniswap.org"
            )));
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

        if candles.len() < limit {
            return Err(CandlesError::InvalidBlockchainData(format!(
                "Only found {} candles, minimum required is {}. Pool may not have enough activity.",
                candles.len(),
                limit
            )));
        }

        Ok(candles)
    }

    fn decode_swap_log(log: &Log) -> Result<(i64, I256, I256), CandlesError> {
        if log.topics().is_empty() || log.data().data.len() < 128 {
            return Err(CandlesError::InvalidBlockchainData("Invalid log format".to_string()));
        }

        let timestamp = log
            .block_timestamp
            .ok_or_else(|| CandlesError::InvalidBlockchainData("Missing block timestamp".to_string()))? as i64
            * 1000;

        let data = &log.data().data;

        let amount0_in_bytes: [u8; 32] = data[0..32]
            .try_into()
            .map_err(|_| CandlesError::InvalidBlockchainData("Failed to parse amount0In".to_string()))?;
        let amount0_in = U256::from_be_bytes(amount0_in_bytes);

        let amount1_in_bytes: [u8; 32] = data[32..64]
            .try_into()
            .map_err(|_| CandlesError::InvalidBlockchainData("Failed to parse amount1In".to_string()))?;
        let amount1_in = U256::from_be_bytes(amount1_in_bytes);

        let amount0_out_bytes: [u8; 32] = data[64..96]
            .try_into()
            .map_err(|_| CandlesError::InvalidBlockchainData("Failed to parse amount0Out".to_string()))?;
        let amount0_out = U256::from_be_bytes(amount0_out_bytes);

        let amount1_out_bytes: [u8; 32] = data[96..128]
            .try_into()
            .map_err(|_| CandlesError::InvalidBlockchainData("Failed to parse amount1Out".to_string()))?;
        let amount1_out = U256::from_be_bytes(amount1_out_bytes);

        let amount0 = if amount0_out > amount0_in {
            I256::try_from(amount0_out - amount0_in).unwrap_or(I256::ZERO)
        } else {
            -I256::try_from(amount0_in - amount0_out).unwrap_or(I256::ZERO)
        };

        let amount1 = if amount1_out > amount1_in {
            I256::try_from(amount1_out - amount1_in).unwrap_or(I256::ZERO)
        } else {
            -I256::try_from(amount1_in - amount1_out).unwrap_or(I256::ZERO)
        };

        Ok((timestamp, amount0, amount1))
    }

    async fn get_token_decimals(provider: &impl Provider<Http<reqwest::Client>>, token_address: Address) -> Result<u8, CandlesError> {
        // ERC20 decimals() function signature
        let decimals_selector = keccak256("decimals()");
        let calldata = decimals_selector[0..4].to_vec();

        let tx = alloy::rpc::types::TransactionRequest::default().to(token_address).input(calldata.into());

        let result = provider.call(&tx).await.map_err(|e| CandlesError::RpcError(format!("Failed to call decimals(): {e}")))?;

        if result.len() < 32 {
            return Err(CandlesError::InvalidBlockchainData("Invalid decimals response".to_string()));
        }

        // decimals returns uint8, but it's padded to 32 bytes
        Ok(result[31])
    }

    async fn get_pool_tokens(provider: &impl Provider<Http<reqwest::Client>>, pool_address: Address) -> Result<(Address, Address), CandlesError> {
        // Uniswap V2 token0() and token1() function signatures
        let token0_selector = keccak256("token0()");
        let token1_selector = keccak256("token1()");

        let token0_calldata = token0_selector[0..4].to_vec();
        let token1_calldata = token1_selector[0..4].to_vec();

        let tx0 = alloy::rpc::types::TransactionRequest::default().to(pool_address).input(token0_calldata.into());

        let tx1 = alloy::rpc::types::TransactionRequest::default().to(pool_address).input(token1_calldata.into());

        let result0 = provider.call(&tx0).await.map_err(|e| CandlesError::RpcError(format!("Failed to call token0(): {e}")))?;

        let result1 = provider.call(&tx1).await.map_err(|e| CandlesError::RpcError(format!("Failed to call token1(): {e}")))?;

        if result0.len() < 32 || result1.len() < 32 {
            return Err(CandlesError::InvalidBlockchainData("Invalid token address response".to_string()));
        }

        // Addresses are returned as 32 bytes with 12 bytes of padding
        let token0 = Address::from_slice(&result0[12..32]);
        let token1 = Address::from_slice(&result1[12..32]);

        Ok((token0, token1))
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
}

#[async_trait]
impl BaseConnection for UniswapV3 {
    async fn get_candles(instrument: Instrument) -> Result<Vec<Candle>, CandlesError> {
        let pool = Pool::try_from(instrument.pair)?;

        let min_candles = std::env::var("UNISWAP_MIN_CANDLES").ok().and_then(|s| s.parse::<usize>().ok()).unwrap_or(250);

        let candles = Self::fetch_and_aggregate_swaps(pool.chain, &pool.pool_address.to_string(), &instrument.timeframe, pool.inverted, min_candles).await?;

        Ok(candles)
    }
}
