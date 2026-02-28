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

impl Timeframe {
    pub fn to_ms(&self) -> i64 {
        match self {
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

#[derive(Debug, Clone)]
pub struct Instrument {
    pub asset_id: String,
    pub asset_symbol: String,
    pub pair: String,
    pub limit: Option<u64>,
    pub start_time: Option<i64>,
    pub end_time: Option<i64>,
    pub connection: Connection,
    pub market_type: MarketType,
    pub timeframe: Timeframe,
}

#[derive(Debug, Serialize, Clone)]
pub struct Candle {
    pub timestamp: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}
