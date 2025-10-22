use serde::Deserialize;

#[derive(Deserialize)]
pub struct HtxKlineResponse {
    pub id: i64,
    pub open: f64,

    pub close: f64,
    pub low: f64,
    pub high: f64,
    pub amount: f64,
}
