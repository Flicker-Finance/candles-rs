#[cfg(test)]
mod test {

    use crate::{
        connections::Connection,
        providers::base::BaseConnection,
        providers::blofin::main::BloFin,
        types::{Instrument, MarketType, Timeframe},
        utils::examine_candles,
    };

    #[tokio::test]
    async fn test_spot_candles() {
        let instrument = Instrument {
            asset_id: "bitcoin".to_owned(),
            pair: "BTC-USDT".to_owned(),
            asset_symbol: "BTC".to_owned(),
            connection: Connection::BloFin,
            market_type: MarketType::Spot,
            timeframe: Timeframe::H1,
            limit: None,
            start_time: None,
            end_time: None,
        };

        match BloFin::get_candles(instrument.clone()).await {
            Ok(result) => examine_candles(&result, instrument),
            Err(err) => panic!("{}", err),
        }
    }

    #[tokio::test]
    async fn test_derivatives_candles() {
        let instrument = Instrument {
            asset_id: "bitcoin".to_owned(),
            pair: "BTC-USDT".to_owned(),
            asset_symbol: "BTC".to_owned(),
            connection: Connection::BloFin,
            market_type: MarketType::Derivatives,
            limit: None,
            start_time: None,
            end_time: None,
            timeframe: Timeframe::H1,
        };

        match BloFin::get_candles(instrument.clone()).await {
            Ok(result) => examine_candles(&result, instrument),
            Err(err) => panic!("{}", err),
        }
    }

    #[tokio::test]
    async fn test_pagination_2000_candles() {
        let instrument = Instrument {
            asset_id: "bitcoin".to_owned(),
            pair: "BTC-USDT".to_owned(),
            asset_symbol: "BTC".to_owned(),
            limit: Some(2000),
            start_time: None,
            end_time: None,
            connection: Connection::BloFin,
            market_type: MarketType::Spot,
            timeframe: Timeframe::H1,
        };

        let candles = instrument.connection.get_candles(instrument.clone()).await.expect("Failed to fetch 2000 candles");

        assert!(candles.len() >= 2000, "Expected at least 2000 candles, got {}", candles.len());

        for i in 1..candles.len() {
            assert!(candles[i].timestamp > candles[i - 1].timestamp, "Candles not in ascending order at index {}", i);
        }
    }
}
