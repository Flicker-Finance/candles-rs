use async_trait::async_trait;

use crate::{
    errors::CandlesError,
    types::{Candle, Instrument},
};

#[async_trait]
pub trait BaseConnection {
    async fn get_candles(instrument: Instrument) -> Result<Vec<Candle>, CandlesError>;
}
