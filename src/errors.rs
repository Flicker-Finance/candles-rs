use thiserror::Error;

#[derive(Error, Debug)]
pub enum CandlesError {
    #[error("Connection not found: {0}")]
    ConnectionNotFound(String),

    #[error("Failed to fetch candles from API: {0}")]
    ApiError(String),

    #[error("Api error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("{0}")]
    Other(String),
}
