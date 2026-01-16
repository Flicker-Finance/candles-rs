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
            timeframe: Timeframe::D1,
        };

        match Hyperliquid::get_candles(instrument.clone()).await {
            Ok(result) => examine_candles(&result, instrument),
            Err(err) => panic!("Failed to fetch candles: {err}"),
        }
    }
}
