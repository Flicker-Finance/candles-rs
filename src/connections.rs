use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

use crate::{
    base::BaseConnection,
    binance::main::Binance,
    bingx::main::BingX,
    blofin::main::BloFin,
    bybit::main::Bybit,
    errors::CandlesError,
    htx::main::HTX,
    mexc::main::Mexc,
    okx::main::OKX,
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
        }
    }
}
