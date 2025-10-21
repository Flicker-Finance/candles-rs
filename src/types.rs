use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

use crate::connections::Connection;

#[derive(Debug, Hash, PartialEq, Eq, Serialize, Deserialize, Default, Clone, Display, EnumString)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum MarketType {
    #[default]
    Spot,
    Derivatives,
}

#[derive(Debug, Hash, Eq, PartialEq, Display, EnumString, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum Timeframe {
    M3,
    M5,
    M15,
    M30,
    H1,
    H4,
    D1,
    W1,
    MN1,
}

#[derive(Debug, Clone)]
pub struct Instrument {
    pub asset_id: String,
    pub pair: String,
    pub connection: Connection,
    pub market_type: MarketType,
    pub timeframe: Timeframe,
}

#[derive(Debug, Serialize, Clone)]
pub struct Candle {
    pub timestamp: i64, // Timestamp in milliseconds
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64, // Volume in base asset (BTC for BTC/USDT)
}
