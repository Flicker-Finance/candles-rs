use async_trait::async_trait;

use crate::{
    errors::CandlesError,
    providers::base::BaseConnection,
    providers::htx::types::HtxKlineResponse,
    types::{Candle, Instrument, MarketType, Timeframe},
    utils::DataWrapper,
};

pub struct HTX;

#[async_trait]
impl BaseConnection for HTX {
    async fn get_candles(instrument: Instrument) -> Result<Vec<Candle>, crate::errors::CandlesError> {
        let htx_timeframe = match instrument.timeframe {
            Timeframe::M3 => return Err(CandlesError::Other("m3 Timeframe is not available for HTX".to_string())),
            Timeframe::M5 => "5min",
            Timeframe::M15 => "15min",
            Timeframe::M30 => "30min",
            Timeframe::H1 => "60min",
            Timeframe::H4 => "4hour",
            Timeframe::D1 => "1day",
            Timeframe::W1 => "1week",
            Timeframe::MN1 => "1mon",
        };

        let url = match instrument.market_type {
            MarketType::Spot => format!(
                "https://api.huobi.pro/market/history/kline?symbol={}&period={}&size=1000",
                instrument.pair.to_lowercase(),
                htx_timeframe
            ),
            MarketType::Derivatives => format!(
                "https://api.hbdm.com/linear-swap-ex/market/history/kline?contract_code={}&period={}&size=1000",
                instrument.pair, htx_timeframe
            ),
        };

        let response: DataWrapper<Vec<HtxKlineResponse>> = reqwest::get(&url).await?.json().await?;

        let iterator: Box<dyn Iterator<Item = _>> = match instrument.market_type {
            MarketType::Spot => Box::new(response.data.into_iter().rev()),
            MarketType::Derivatives => Box::new(response.data.into_iter()),
        };

        Ok(iterator
            .map(|f| Candle {
                timestamp: f.id * 1000,
                open: f.open,
                high: f.high,
                low: f.low,
                close: f.close,
                volume: f.amount,
            })
            .collect())
    }
}
