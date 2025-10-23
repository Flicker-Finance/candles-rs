use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

use crate::{
    errors::CandlesError,
    providers::base::BaseConnection,
    providers::binance::main::Binance,
    providers::bingx::main::BingX,
    providers::blofin::main::BloFin,
    providers::bybit::main::Bybit,
    providers::coingecko::main::CoinGecko,
    providers::htx::main::HTX,
    providers::mexc::main::Mexc,
    providers::moralis::main::Moralis,
    providers::okx::main::OKX,
    providers::uniswap_v3::main::UniswapV3,
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

    UniswapV3,
    Moralis,
    CoinGecko,
}

impl Connection {
    pub async fn get_candles(&self, instrument: Instrument) -> Result<Vec<Candle>, CandlesError> {
        match self {
            Connection::Binance => Binance::get_candles(instrument).await,
            Connection::OKX => OKX::get_candles(instrument).await,
            Connection::BloFin => BloFin::get_candles(instrument).await,
            Connection::Bybit => Bybit::get_candles(instrument).await,
            Connection::BingX => BingX::get_candles(instrument).await,
            Connection::HTX => HTX::get_candles(instrument).await,
            Connection::Mexc => Mexc::get_candles(instrument).await,
            Connection::UniswapV3 => UniswapV3::get_candles(instrument).await,
            Connection::Moralis => Moralis::get_candles(instrument).await,
            Connection::CoinGecko => CoinGecko::get_candles(instrument).await,
        }
    }
}
