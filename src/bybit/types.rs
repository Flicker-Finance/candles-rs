use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
pub struct BybitKlineResponse {
    pub list: Vec<Value>,
}
