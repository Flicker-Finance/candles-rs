use thiserror::Error;

#[derive(Error, Debug)]
pub enum CandlesError {
    #[error("Connection not found: {0}")]
    ConnectionNotFound(String),

    #[error("Failed to fetch candles from API: {0}")]
    ApiError(String),

    #[error("HTTP request failed with status {status}: {body}")]
    HttpError { status: u16, body: String },

    #[error("Failed to parse JSON response: {0}")]
    JsonParseError(String),

    #[error("Invalid data format at index {index}: {message}")]
    InvalidDataFormat { index: usize, message: String },

    #[error("Missing required field '{field}' at index {index}")]
    MissingField { field: String, index: usize },

    #[error("Failed to parse {field}: {message}")]
    ParseError { field: String, message: String },

    #[error("Invalid address format: {0}")]
    InvalidAddress(String),

    #[error("Invalid pair format: {0}")]
    InvalidPairFormat(String),

    #[error("Unsupported timeframe '{timeframe}' for provider '{provider}'")]
    UnsupportedTimeframe { timeframe: String, provider: String },

    #[error("Missing environment variable: {0}")]
    MissingEnvVar(String),

    #[error("RPC call failed: {0}")]
    RpcError(String),

    #[error("Invalid blockchain data: {0}")]
    InvalidBlockchainData(String),

    #[error("Api error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("Strum Parse error: {0}")]
    StrumParse(#[from] strum::ParseError),

    #[error("{0}")]
    Other(String),
}
