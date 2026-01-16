use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug)]
pub struct CandleSnapshotRequest {
    #[serde(rename = "type")]
    pub request_type: String,
    pub req: CandleSnapshotParams,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CandleSnapshotParams {
    pub coin: String,
    pub interval: String,
    pub start_time: i64,
    pub end_time: i64,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(dead_code)]
pub struct HyperliquidCandle {
    /// Open timestamp (millis)
    pub t: i64,
    /// Close timestamp (millis)
    #[serde(rename = "T")]
    pub close_time: i64,
    /// Symbol
    pub s: String,
    /// Interval
    pub i: String,
    /// Open price
    pub o: String,
    /// Close price
    pub c: String,
    /// High price
    pub h: String,
    /// Low price
    pub l: String,
    /// Volume
    pub v: String,
    /// Number of trades
    pub n: i64,
}
