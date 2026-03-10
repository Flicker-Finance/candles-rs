use crate::types::Instrument;
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
        Value::String(s) => s.parse().map_err(|_| CandlesError::ParseError {
            field: field.to_string(),
            message: format!("at index {index}: {val}"),
        }),
        Value::Number(n) => n.as_f64().ok_or_else(|| CandlesError::ParseError {
            field: format!("{field} to f64"),
            message: format!("at index {index}: {val}"),
        }),
        _ => Err(CandlesError::InvalidDataFormat {
            index,
            message: format!("Invalid {field} type: expected string or number, got {val}"),
        }),
    }
}

pub fn examine_candles(candles: &[Candle], instrument: Instrument) {
    assert!(!candles.is_empty(), "Candles array is empty");
    let expected_min = instrument.limit.unwrap_or(50) as usize;
    assert!(candles.len() >= expected_min, "Expected at least {} candles, got {}", expected_min, candles.len());

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

    println!(
        "candles: count={}, first_ts={}, last_ts={}",
        candles.len(),
        candles.first().unwrap().timestamp,
        candles.last().unwrap().timestamp
    );

    let candle = candles.last().unwrap();

    assert!(
        DateTime::from_timestamp_millis(candle.timestamp).is_some(),
        "Timestamp {} is not valid milliseconds",
        candle.timestamp
    );

    let candle_time = DateTime::from_timestamp_millis(candle.timestamp).unwrap();
    let now = Utc::now();
    assert!(
        candle_time - Duration::seconds(3) <= now,
        "Timestamp {}({}) is in the future, while now is {}",
        candle_time,
        candle.timestamp,
        now
    );

    assert!(
        candle_time.year() == now.year(),
        "Timestamp year {} should be current year {}",
        candle_time.year(),
        now.year()
    );

    assert!(candle.high >= candle.low, "High ({}) should be >= low ({})", candle.high, candle.low);
    assert!(candle.close > 0.0, "Close price {} should be positive", candle.close);
    assert!(candle.volume >= 0.0, "Volume {} should be non-negative", candle.volume);

    // Volume should be in base asset, not quote. If volume >> close, it's likely quote volume.
    if candle.close >= 1.0 {
        assert!(
            candle.volume < candle.close * 1_000_000.0,
            "Volume ({}) looks like quote asset volume (close: {}). Should be in base asset.",
            candle.volume,
            candle.close
        );
    }
}
