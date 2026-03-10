#[cfg(test)]
mod test {

    use crate::{
        connections::Connection,
        providers::base::BaseConnection,
        providers::freedx::main::FreeDX,
        types::{Instrument, MarketType, Timeframe},
        utils::examine_candles,
    };

    #[tokio::test]
    async fn test_spot_candles() {
        let instrument = Instrument {
            asset_id: "bitcoin".to_owned(),
            pair: "BTC-USDT".to_owned(),
            asset_symbol: "BTC".to_owned(),
            limit: None,
            start_time: None,
            end_time: None,
            connection: Connection::Freedx,
            market_type: MarketType::Spot,
            timeframe: Timeframe::H1,
        };

        match FreeDX::get_candles(instrument.clone()).await {
            Ok(result) => examine_candles(&result, instrument),
            Err(err) => panic!("{}", err),
        }
    }

    #[tokio::test]
    async fn test_pagination_candles() {
        let instrument = Instrument {
            asset_id: "bitcoin".to_owned(),
            pair: "BTC-USDT".to_owned(),
            asset_symbol: "BTC".to_owned(),
            limit: Some(2000),
            start_time: None,
            end_time: None,
            connection: Connection::Freedx,
            market_type: MarketType::Spot,
            timeframe: Timeframe::H1,
        };

        match Connection::Freedx.get_candles(instrument.clone()).await {
            Ok(result) => examine_candles(&result, instrument),
            Err(err) => panic!("{}", err),
        }
    }

    #[tokio::test]
    async fn test_derivatives_candles() {
        let instrument = Instrument {
            asset_id: "bitcoin".to_owned(),
            pair: "BTC-PERP".to_owned(),
            asset_symbol: "BTC".to_owned(),
            limit: None,
            start_time: None,
            end_time: None,
            connection: Connection::Freedx,
            market_type: MarketType::Derivatives,
            timeframe: Timeframe::H1,
        };

        match FreeDX::get_candles(instrument.clone()).await {
            Ok(result) => examine_candles(&result, instrument),
            Err(err) => panic!("{}", err),
        }
    }
}
