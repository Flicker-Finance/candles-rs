use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct TokenPriceResponse {
    #[serde(rename = "tokenName")]
    pub token_name: String,

    #[serde(rename = "tokenSymbol")]
    pub token_symbol: String,

    #[serde(rename = "tokenLogo")]
    pub token_logo: Option<String>,

    #[serde(rename = "tokenDecimals")]
    pub token_decimals: String,

    #[serde(rename = "nativePrice")]
    pub native_price: Option<NativePrice>,

    #[serde(rename = "usdPrice")]
    #[serde(deserialize_with = "deserialize_number_from_any")]
    pub usd_price: f64,

    #[serde(rename = "usdPriceFormatted")]
    pub usd_price_formatted: String,

    #[serde(rename = "exchangeAddress")]
    pub exchange_address: Option<String>,

    #[serde(rename = "exchangeName")]
    pub exchange_name: Option<String>,

    #[serde(rename = "tokenAddress")]
    pub token_address: String,

    #[serde(rename = "24hrPercentChange")]
    pub percent_change_24h: Option<String>,

    #[serde(rename = "securityScore")]
    pub security_score: Option<u8>,

    #[serde(rename = "pairAddress")]
    pub pair_address: Option<String>,

    #[serde(rename = "pairTotalLiquidityUsd")]
    pub pair_total_liquidity_usd: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct NativePrice {
    pub value: String,
    pub decimals: u8,
    pub name: String,
    pub symbol: String,
}

#[derive(Deserialize, Debug)]
pub struct MoralisOhlcvResponse {
    // #[serde(default)]
    // pub cursor: Option<serde_json::Value>,

    // #[serde(default)]
    // pub page: Option<serde_json::Value>,

    // #[serde(rename = "pairAddress")]
    // pub pair_address: String,

    // #[serde(rename = "tokenAddress")]
    // pub token_address: String,

    // pub timeframe: String,
    // pub currency: String,
    pub result: Vec<MoralisCandle>,
}

#[derive(Deserialize, Debug)]
pub struct MoralisCandle {
    pub timestamp: String,

    #[serde(deserialize_with = "deserialize_number_from_any")]
    pub open: f64,

    #[serde(deserialize_with = "deserialize_number_from_any")]
    pub high: f64,

    #[serde(deserialize_with = "deserialize_number_from_any")]
    pub low: f64,

    #[serde(deserialize_with = "deserialize_number_from_any")]
    pub close: f64,

    #[serde(deserialize_with = "deserialize_number_from_any")]
    pub volume: f64,
}

fn deserialize_number_from_any<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    use serde_json::Value;

    let value = Value::deserialize(deserializer)?;
    match value {
        Value::Number(n) => n.as_f64().ok_or_else(|| Error::custom("Invalid number")),
        Value::String(s) => s.parse::<f64>().map_err(|e| Error::custom(format!("Failed to parse string as f64: {e}"))),
        _ => Err(Error::custom("Expected number or string")),
    }
}
