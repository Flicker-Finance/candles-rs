use crate::{errors::CandlesError, types::Candle};
use chrono::Utc;
use chrono::{DateTime, Datelike, Duration};
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize, Debug)]
pub struct DataWrapper<T> {
    pub data: T,
}

#[derive(Deserialize, Debug)]
pub struct ResultWrapper<T> {
    pub result: T,
}

#[derive(Deserialize, Debug)]
pub struct DataWrapperWithMsgCode<C, T> {
    pub code: C,
    pub msg: Option<String>,
    pub data: T,
}

#[derive(Deserialize, Debug)]
pub struct DataWrapperWithStatusCode<C, T> {
    pub code: C,
    pub message: Option<String>,
    pub data: T,
}

pub fn parse_string_to_f64(val: &Value, field: &str, index: usize) -> Result<f64, CandlesError> {
    match val {
        // Handle string values like "4524.78"
        Value::String(s) => s.parse().map_err(|_| CandlesError::ParseError {
            field: field.to_string(),
            message: format!("at index {index}: {val}"),
        }),
        // Handle number values like 4524.78
        Value::Number(n) => n.as_f64().ok_or_else(|| CandlesError::ParseError {
            field: format!("{field} to f64"),
            message: format!("at index {index}: {val}"),
        }),
        // Handle any other type
        _ => Err(CandlesError::InvalidDataFormat {
            index,
            message: format!("Invalid {field} type: expected string or number, got {val}"),
        }),
    }
}

pub fn examine_candles(candles: &[Candle]) {
    assert!(!candles.is_empty(), "Candles array is empty");
    assert!(candles.len() >= 5, "Candles length is < 5");

    // Check all candles are in ascending order (oldest to newest)
    for i in 1..candles.len() {
        assert!(
            candles[i].timestamp > candles[i - 1].timestamp,
            "Candles are not in ascending order: candle at index {} ({}) should be after candle at index {} ({})",
            i,
            candles[i].timestamp,
            i - 1,
            candles[i - 1].timestamp
        );
    }

    // Pick the last candle and do ordinary checks
    let candle = candles.last().unwrap();

    // Check timestamp is valid milliseconds by attempting to parse
    assert!(
        DateTime::from_timestamp_millis(candle.timestamp).is_some(),
        "Timestamp {} is not valid milliseconds",
        candle.timestamp
    );

    // Check timestamp is not in the future
    let candle_time = DateTime::from_timestamp_millis(candle.timestamp).unwrap();
    let now = Utc::now();
    assert!(
        candle_time - Duration::seconds(3) <= now,
        "Timestamp {}({}) is in the future, while now is {}",
        candle_time,
        candle.timestamp,
        now
    );

    // Check timestamp year is current year
    assert!(
        candle_time.year() == now.year(),
        "Timestamp year {} should be current year {}",
        candle_time.year(),
        now.year()
    );

    // Check high >= low
    assert!(candle.high >= candle.low, "High ({}) should be >= low ({})", candle.high, candle.low);

    // Check close exists and is valid
    assert!(candle.close > 0.0, "Close price {} should be positive", candle.close);

    // Check volume exists
    assert!(candle.volume >= 0.0, "Volume {} should be non-negative", candle.volume);
}
