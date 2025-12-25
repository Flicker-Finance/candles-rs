use std::env;

use chrono::NaiveDateTime;

use crate::errors::CandlesError;

pub fn get_api_key() -> Result<String, CandlesError> {
    env::var("ALPHA_VANTAGE_API_KEY").map_err(|_| CandlesError::MissingEnvVar("ALPHA_VANTAGE_API_KEY".to_string()))
}

pub fn parse_f64(value: &str, field: &str, date: &str) -> Result<f64, CandlesError> {
    value.parse::<f64>().map_err(|_| CandlesError::ParseError {
        field: field.to_string(),
        message: format!("Failed to parse '{value}' at date {date}"),
    })
}

pub fn parse_timestamp(date_str: &str, has_time: bool) -> Result<i64, CandlesError> {
    let datetime_str = if has_time { date_str.to_string() } else { format!("{date_str} 00:00:00") };

    NaiveDateTime::parse_from_str(&datetime_str, "%Y-%m-%d %H:%M:%S")
        .map(|dt| dt.and_utc().timestamp_millis())
        .map_err(|e| CandlesError::ParseError {
            field: "timestamp".to_string(),
            message: format!("Failed to parse date '{date_str}': {e}"),
        })
}
