use chrono::Utc;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

use crate::{
    errors::CandlesError,
    providers::alpha_vantage::main::AlphaVantage,
    providers::base::BaseConnection,
    providers::binance::main::Binance,
    providers::bingx::main::BingX,
    providers::blofin::main::BloFin,
    providers::bybit::main::Bybit,
    providers::coingecko::main::CoinGecko,
    providers::freedx::main::FreeDX,
    providers::htx::main::HTX,
    providers::hyperliquid::main::Hyperliquid,
    providers::massive::main::Massive,
    providers::mexc::main::Mexc,
    providers::okx::main::OKX,
    types::{Candle, Instrument},
};

#[derive(Hash, PartialEq, Eq, Debug, Display, EnumString, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum Connection {
    Binance,
    OKX,
    BloFin,
    Bybit,
    BingX,
    HTX,
    Mexc,
    Hyperliquid,
    Freedx,

    CoinGecko,

    AlphaVantage,
    Massive,
}

const MAX_PAGINATION_ITERATIONS: usize = 50;

impl Connection {
    pub async fn get_candles(&self, instrument: Instrument) -> Result<Vec<Candle>, CandlesError> {
        let max_batch = match self.max_batch_size() {
            Some(batch) => batch,
            None => return self.fetch_single_batch(instrument).await,
        };

        let desired_limit = instrument.limit.unwrap_or(max_batch);

        if desired_limit <= max_batch && instrument.start_time.is_none() {
            return self.fetch_single_batch(instrument).await;
        }

        let mut batches: Vec<Vec<Candle>> = Vec::new();
        let mut total_count: u64 = 0;
        let mut current_end_time = instrument.end_time.unwrap_or_else(|| Utc::now().timestamp_millis());
        let start_time_bound = instrument.start_time;

        for _ in 0..MAX_PAGINATION_ITERATIONS {
            let batch_instrument = Instrument {
                asset_id: instrument.asset_id.clone(),
                asset_symbol: instrument.asset_symbol.clone(),
                pair: instrument.pair.clone(),
                limit: Some(max_batch),
                start_time: start_time_bound,
                end_time: Some(current_end_time),
                connection: instrument.connection.clone(),
                market_type: instrument.market_type.clone(),
                timeframe: instrument.timeframe.clone(),
            };

            let batch = self.fetch_single_batch(batch_instrument).await?;

            if batch.is_empty() {
                break;
            }

            let batch_len = batch.len() as u64;
            let oldest_timestamp = batch[0].timestamp;

            let newest_existing = batches.last().and_then(|b: &Vec<Candle>| b.first()).map(|c| c.timestamp);
            let new_candles: Vec<Candle> = if let Some(newest) = newest_existing {
                batch.into_iter().filter(|c| c.timestamp < newest).collect()
            } else {
                batch
            };

            if new_candles.is_empty() {
                break;
            }

            total_count += new_candles.len() as u64;
            batches.push(new_candles);

            let have_enough = total_count >= desired_limit;
            let reached_start = start_time_bound.is_some_and(|start| oldest_timestamp <= start);
            let exhausted = batch_len < max_batch;

            if have_enough || reached_start || exhausted {
                break;
            }

            current_end_time = oldest_timestamp - 1;
        }

        batches.reverse();
        let mut all_candles: Vec<Candle> = batches.into_iter().flatten().collect();

        if let Some(start) = start_time_bound {
            all_candles.retain(|c| c.timestamp >= start);
        }

        if all_candles.len() as u64 > desired_limit {
            let skip = all_candles.len() - desired_limit as usize;
            all_candles = all_candles.into_iter().skip(skip).collect();
        }

        Ok(all_candles)
    }

    fn max_batch_size(&self) -> Option<u64> {
        match self {
            Connection::Binance => Some(1000),
            Connection::OKX => Some(100),
            Connection::Bybit => Some(1000),
            Connection::BloFin => Some(100),
            Connection::BingX => Some(1000),
            Connection::HTX => Some(2000),
            Connection::Mexc => Some(500),
            Connection::Hyperliquid => Some(5000),
            Connection::Freedx => Some(300),
            Connection::CoinGecko => Some(1000),
            Connection::AlphaVantage => None,
            Connection::Massive => Some(50_000),
        }
    }

    async fn fetch_single_batch(&self, instrument: Instrument) -> Result<Vec<Candle>, CandlesError> {
        match self {
            Connection::Binance => Binance::get_candles(instrument).await,
            Connection::OKX => OKX::get_candles(instrument).await,
            Connection::BloFin => BloFin::get_candles(instrument).await,
            Connection::Bybit => Bybit::get_candles(instrument).await,
            Connection::BingX => BingX::get_candles(instrument).await,
            Connection::HTX => HTX::get_candles(instrument).await,
            Connection::Mexc => Mexc::get_candles(instrument).await,
            Connection::Hyperliquid => Hyperliquid::get_candles(instrument).await,
            Connection::Freedx => FreeDX::get_candles(instrument).await,
            Connection::CoinGecko => CoinGecko::get_candles(instrument).await,
            Connection::AlphaVantage => AlphaVantage::get_candles(instrument).await,
            Connection::Massive => Massive::get_candles(instrument).await,
        }
    }
}
