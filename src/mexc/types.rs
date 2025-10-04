use serde::Deserialize;

#[derive(Deserialize)]
pub struct MexcKlineFuturesResponse {
    pub time: Vec<i64>,
    pub open: Vec<f64>,

    pub close: Vec<f64>,
    pub low: Vec<f64>,
    pub high: Vec<f64>,
    pub vol: Vec<f64>,
}
