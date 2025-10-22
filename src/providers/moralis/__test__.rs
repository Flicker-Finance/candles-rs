#[cfg(test)]
mod test {
    use crate::{
        connections::Connection,
        providers::base::BaseConnection,
        providers::moralis::main::Moralis,
        types::{Instrument, MarketType, Timeframe},
        utils::examine_candles,
    };

    #[tokio::test]
    #[ignore = "MORALIS_API_KEY needed"]
    async fn test_moralis_ethereum_usdc_weth_15m() {
        // USDC/WETH pool on Ethereum
        let instrument = Instrument {
            asset_id: "ethereum_usdc_weth".to_owned(),
            pair: "eth_0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640".to_owned(),
            connection: Connection::Moralis,
            market_type: MarketType::Spot,
            timeframe: Timeframe::M15,
        };

        match Moralis::get_candles(instrument).await {
            Ok(result) => examine_candles(&result),
            Err(err) => panic!("Failed to fetch candles: {err}"),
        }
    }

    #[tokio::test]
    #[ignore = "MORALIS_API_KEY needed"]
    async fn test_moralis_base_pool_1h() {
        // Test pool on Base chain
        let instrument = Instrument {
            asset_id: "base_test_pool".to_owned(),
            pair: "base_0xE1BeD6AAdBa5471700f16A47EEe2504346B724aD".to_owned(),
            connection: Connection::Moralis,
            market_type: MarketType::Spot,
            timeframe: Timeframe::H1,
        };

        match Moralis::get_candles(instrument).await {
            Ok(result) => examine_candles(&result),
            Err(err) => panic!("Failed to fetch candles: {err}"),
        }
    }
}
