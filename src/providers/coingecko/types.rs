use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct OhlcvResponse {
    pub data: OhlcvData,
}

#[derive(Deserialize, Debug, Clone)]
pub struct OhlcvData {
    pub id: String,
    pub r#type: String,
    pub attributes: OhlcvAttributes,
}

#[derive(Deserialize, Debug, Clone)]
pub struct OhlcvAttributes {
    pub ohlcv_list: Vec<Vec<serde_json::Value>>,
}
