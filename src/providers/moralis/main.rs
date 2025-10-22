use alloy::primitives::Address;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::str::FromStr;

use crate::{
    errors::CandlesError,
    modules::chains::Chain,
    modules::pairs::parse_pair,
    providers::{
        base::BaseConnection,
        moralis::types::{MoralisOhlcvResponse, TokenPriceResponse},
    },
    types::{Candle, Instrument, Timeframe},
};

pub struct Moralis;

impl Moralis {
    fn get_moralis_timeframe(timeframe: &Timeframe) -> Result<&'static str, CandlesError> {
        match timeframe {
            Timeframe::M3 => Err(CandlesError::UnsupportedTimeframe {
                timeframe: "3m".to_string(),
                provider: "Moralis".to_string(),
            }),
            Timeframe::M5 => Ok("5min"),
            Timeframe::M15 => Ok("15min"),
            Timeframe::M30 => Ok("30min"),
            Timeframe::H1 => Ok("1h"),
            Timeframe::H4 => Ok("4h"),
            Timeframe::D1 => Ok("1d"),
            Timeframe::W1 => Err(CandlesError::UnsupportedTimeframe {
                timeframe: "1w".to_string(),
                provider: "Moralis".to_string(),
            }),
            Timeframe::MN1 => Err(CandlesError::UnsupportedTimeframe {
                timeframe: "1M".to_string(),
                provider: "Moralis".to_string(),
            }),
        }
    }

    /// Fetches current token price information from Moralis API
    ///
    /// # Arguments
    /// * `token_address` - The contract address of the token
    /// * `chain` - Optional blockchain to query. If None, Moralis will auto-detect
    ///
    /// # Example
    /// ```ignore
    /// use candles_rs::providers::moralis::Moralis;
    /// use candles_rs::modules::chains::Chain;
    ///
    /// let price = Moralis::get_token_price(
    ///     "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
    ///     Some(Chain::Ethereum)
    /// ).await?;
    ///
    /// println!("Price: ${}", price.usd_price);
    /// ```
    pub async fn get_token_price(token_address: &str, chain: Option<Chain>) -> Result<TokenPriceResponse, CandlesError> {
        Address::from_str(token_address).map_err(|_| CandlesError::InvalidAddress(token_address.to_string()))?;

        let api_key = std::env::var("MORALIS_API_KEY").map_err(|_| CandlesError::MissingEnvVar("MORALIS_API_KEY".to_string()))?;

        let url = format!("https://deep-index.moralis.io/api/v2.2/erc20/{token_address}/price");

        let client = reqwest::Client::new();
        let mut request = client.get(&url).header("X-API-Key", &api_key);

        if let Some(chain_value) = chain {
            request = request.query(&[("chain", chain_value.as_str_moralis())]);
        }

        let response = request.send().await.map_err(|e| CandlesError::ApiError(format!("Failed to fetch token price: {e}")))?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            return Err(CandlesError::HttpError { status, body });
        }

        let price_info: TokenPriceResponse = response
            .json()
            .await
            .map_err(|e| CandlesError::JsonParseError(format!("Failed to parse Moralis token price response: {e}")))?;

        Ok(price_info)
    }
}

#[async_trait]
impl BaseConnection for Moralis {
    async fn get_candles(instrument: Instrument) -> Result<Vec<Candle>, CandlesError> {
        let api_key = std::env::var("MORALIS_API_KEY").map_err(|_| CandlesError::MissingEnvVar("MORALIS_API_KEY".to_string()))?;

        let (chain, pair_address, _inverted) = parse_pair(&instrument.pair)?;
        let timeframe = Self::get_moralis_timeframe(&instrument.timeframe)?;

        let now = Utc::now();
        let from_date = now - chrono::Duration::days(30);
        let from_date_str = from_date.format("%Y-%m-%dT%H:%M:%S%.3f").to_string();
        let to_date_str = now.format("%Y-%m-%dT%H:%M:%S%.3f").to_string();

        let url = format!("https://deep-index.moralis.io/api/v2.2/pairs/{pair_address}/ohlcv");

        let client = reqwest::Client::new();
        let response: MoralisOhlcvResponse = client
            .get(&url)
            .header("X-API-Key", api_key)
            .query(&[
                ("chain", chain.as_str_moralis()),
                ("timeframe", timeframe),
                ("currency", "usd"),
                ("fromDate", from_date_str.as_str()),
                ("toDate", to_date_str.as_str()),
                ("limit", "300"),
            ])
            .send()
            .await?
            .json()
            .await?;

        let mut candles: Vec<Candle> = response
            .result
            .into_iter()
            .map(|candle| {
                let timestamp_ms = DateTime::parse_from_rfc3339(&candle.timestamp)
                    .map_err(|e| CandlesError::ParseError {
                        field: "timestamp".to_string(),
                        message: e.to_string(),
                    })?
                    .timestamp_millis();

                Ok(Candle {
                    timestamp: timestamp_ms,
                    open: candle.open,
                    high: candle.high,
                    low: candle.low,
                    close: candle.close,
                    volume: candle.volume,
                })
            })
            .collect::<Result<Vec<Candle>, CandlesError>>()?;

        candles.reverse();

        Ok(candles)
    }
}
