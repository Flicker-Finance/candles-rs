#[cfg(test)]
mod test {

    use crate::{
        base::BaseConnection,
        binance::main::Binance,
        connections::Connection,
        types::{Instrument, MarketType, Timeframe},
    };

    #[tokio::test]
    async fn test_spot_candles() {
        let instrument = Instrument {
            asset_id: "bitcoin".to_owned(),
            pair: "BTCUSDT".to_owned(),
            connection: Connection::Binance,
            market_type: MarketType::Spot,
            timeframe: Timeframe::H1,
        };

        match Binance::get_candles(instrument).await {
            Ok(result) => assert!(result.len() >= 5, "Candles length is < 5"),
            Err(err) => panic!("{}", err),
        }
    }

    #[tokio::test]
    async fn test_derivatives_candles() {
        let instrument = Instrument {
            asset_id: "bitcoin".to_owned(),
            pair: "BTCUSDT".to_owned(),
            connection: Connection::Binance,
            market_type: MarketType::Derivatives,
            timeframe: Timeframe::H1,
        };

        match Binance::get_candles(instrument).await {
            Ok(result) => assert!(result.len() >= 5, "Candles length is < 5"),
            Err(err) => panic!("{}", err),
        }
    }
}
