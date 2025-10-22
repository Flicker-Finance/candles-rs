#[cfg(test)]
mod test {

    use crate::{
        connections::Connection,
        providers::base::BaseConnection,
        providers::okx::main::OKX,
        types::{Instrument, MarketType, Timeframe},
        utils::examine_candles,
    };

    #[tokio::test]
    async fn test_spot_candles() {
        let instrument = Instrument {
            asset_id: "bitcoin".to_owned(),
            pair: "BTC-USDT".to_owned(),
            connection: Connection::OKX,
            market_type: MarketType::Spot,
            timeframe: Timeframe::H1,
        };

        match OKX::get_candles(instrument).await {
            Ok(result) => examine_candles(&result),
            Err(err) => panic!("{}", err),
        }
    }

    #[tokio::test]
    async fn test_derivatives_candles() {
        let instrument = Instrument {
            asset_id: "bitcoin".to_owned(),
            pair: "BTC-USDT".to_owned(),
            connection: Connection::OKX,
            market_type: MarketType::Derivatives,
            timeframe: Timeframe::H1,
        };

        match OKX::get_candles(instrument).await {
            Ok(result) => examine_candles(&result),
            Err(err) => panic!("{}", err),
        }
    }
}
