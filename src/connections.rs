use serde::{Deserialize, Serialize};

use crate::{
    base::BaseConnection,
    binance::main::Binance,
    bingx::main::BingX,
    blofin::main::BloFin,
    bybit::main::Bybit,
    errors::CandlesError,
    htx::main::HTX,
    okx::main::OKX,
    types::{Candle, Instrument},
};

#[derive(Hash, PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Connection {
    Binance,
    OKX,
    BloFin,
    Bybit,
    BingX,
    HTX,
}

impl TryFrom<&str> for Connection {
    type Error = CandlesError; // You need to define the Error type

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "binance" => Ok(Connection::Binance),
            "okx" => Ok(Connection::OKX),
            "blofin" => Ok(Connection::BloFin),
            "bybit" => Ok(Connection::Bybit),
            "bingx" => Ok(Connection::BingX),
            "htx" => Ok(Connection::HTX),
            _ => Err(CandlesError::ConnectionNotFound(value.to_string())),
        }
    }
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
        }
    }

    pub fn get_connection_id(&self) -> String {
        match self {
            Connection::Binance => "binance".to_owned(),
            Connection::OKX => "okx".to_owned(),
            Connection::BloFin => "blofin".to_owned(),
            Connection::Bybit => "bybit".to_owned(),
            Connection::BingX => "bingx".to_owned(),
            Connection::HTX => "htx".to_owned(),
        }
    }
}
