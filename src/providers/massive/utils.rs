use std::env;

use crate::{errors::CandlesError, types::Timeframe};

pub fn get_api_key() -> Result<String, CandlesError> {
    env::var("MASSIVE_API_KEY").map_err(|_| CandlesError::MissingEnvVar("MASSIVE_API_KEY".to_string()))
}

/// Map a `Timeframe` to Massive's `(multiplier, timespan)` URL pair.
/// Massive's intraday timespans are minute/hour and require a multiplier.
pub fn timeframe_to_massive(tf: &Timeframe) -> Result<(u32, &'static str), CandlesError> {
    match tf {
        Timeframe::M5 => Ok((5, "minute")),
        Timeframe::M15 => Ok((15, "minute")),
        Timeframe::M30 => Ok((30, "minute")),
        Timeframe::H1 => Ok((1, "hour")),
        Timeframe::H4 => Ok((4, "hour")),
        Timeframe::D1 => Ok((1, "day")),
        Timeframe::W1 => Ok((1, "week")),
        Timeframe::MN1 => Ok((1, "month")),
        // Massive doesn't offer 3-minute aggregates.
        Timeframe::M3 => Err(CandlesError::UnsupportedTimeframe {
            timeframe: format!("{tf:?}"),
            provider: "Massive".to_string(),
        }),
    }
}
