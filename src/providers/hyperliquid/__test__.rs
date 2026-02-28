#[cfg(test)]
mod test {
    use crate::{
        connections::Connection,
        providers::base::BaseConnection,
        providers::hyperliquid::main::Hyperliquid,
        types::{Instrument, MarketType, Timeframe},
        utils::examine_candles,
    };

    #[tokio::test]
    async fn test_hyperliquid_btc_1h() {
        let instrument = Instrument {
            asset_id: "BTC".to_owned(),
            pair: "BTC".to_owned(),
            asset_symbol: "BTC".to_owned(),
            connection: Connection::Hyperliquid,
            market_type: MarketType::Derivatives,
            limit: None,
            start_time: None,
            end_time: None,
            timeframe: Timeframe::H1,
        };

        match Hyperliquid::get_candles(instrument.clone()).await {
            Ok(result) => examine_candles(&result, instrument),
            Err(err) => panic!("Failed to fetch candles: {err}"),
        }
    }

    #[tokio::test]
    async fn test_hyperliquid_eth_15m() {
        let instrument = Instrument {
            asset_id: "ETH".to_owned(),
            pair: "ETH".to_owned(),
            asset_symbol: "ETH".to_owned(),
            connection: Connection::Hyperliquid,
            market_type: MarketType::Derivatives,
            limit: None,
            start_time: None,
            end_time: None,
            timeframe: Timeframe::M15,
        };

        match Hyperliquid::get_candles(instrument.clone()).await {
            Ok(result) => examine_candles(&result, instrument),
            Err(err) => panic!("Failed to fetch candles: {err}"),
        }
    }

    #[tokio::test]
    async fn test_hyperliquid_sol_1d() {
        let instrument = Instrument {
            asset_id: "SOL".to_owned(),
            pair: "SOL".to_owned(),
            asset_symbol: "SOL".to_owned(),
            connection: Connection::Hyperliquid,
            market_type: MarketType::Derivatives,
            limit: None,
            start_time: None,
            end_time: None,
            timeframe: Timeframe::D1,
        };

        match Hyperliquid::get_candles(instrument.clone()).await {
            Ok(result) => examine_candles(&result, instrument),
            Err(err) => panic!("Failed to fetch candles: {err}"),
        }
    }

    #[tokio::test]
    async fn test_pagination_2000_candles() {
        let instrument = Instrument {
            asset_id: "BTC".to_owned(),
            pair: "BTC".to_owned(),
            asset_symbol: "BTC".to_owned(),
            limit: Some(2000),
            start_time: None,
            end_time: None,
            connection: Connection::Hyperliquid,
            market_type: MarketType::Derivatives,
            timeframe: Timeframe::H1,
        };

        let candles = instrument.connection.get_candles(instrument.clone()).await.expect("Failed to fetch 2000 candles");

        assert!(candles.len() >= 2000, "Expected at least 2000 candles, got {}", candles.len());

        for i in 1..candles.len() {
            assert!(candles[i].timestamp > candles[i - 1].timestamp, "Candles not in ascending order at index {}", i);
        }
    }
}
