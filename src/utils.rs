use serde::Deserialize;
use serde_json::Value;

use crate::errors::CandlesError;

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
        Value::String(s) => s.parse().map_err(|_| CandlesError::Other(format!("Failed to parse {field} at index {index}: {val}"))),
        // Handle number values like 4524.78
        Value::Number(n) => n
            .as_f64()
            .ok_or_else(|| CandlesError::Other(format!("Failed to convert {field} to f64 at index {index}: {val}"))),
        // Handle any other type
        _ => Err(CandlesError::Other(format!("Invalid {field} type at index {index}: expected string or number, got {val}"))),
    }
}
