#[cfg(test)]
mod test {
    use crate::{
        connections::Connection,
        providers::base::BaseConnection,
        providers::uniswap_v3::main::UniswapV3,
        types::{Instrument, MarketType, Timeframe},
        utils::examine_candles,
    };

    #[tokio::test]
    #[ignore = "Takes too long"]
    async fn test_uniswap_v3_ethereum_usdc_weth_m15() {
        unsafe {
            std::env::set_var("UNISWAP_BATCH_SIZE", "1000");
        }

        let instrument = Instrument {
            asset_id: "ethereum_usdc_weth".to_owned(),
            pair: "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2_ethereum_0x4e68Ccd3E89f51C3074ca5072bbAC773960dFa36".to_owned(),
            connection: Connection::UniswapV3,
            market_type: MarketType::Spot,
            timeframe: Timeframe::M15,
            limit: Some(5),
        };

        match UniswapV3::get_candles(instrument.clone()).await {
            Ok(result) => examine_candles(&result, instrument),
            Err(err) => panic!("Failed to fetch candles: {err}"),
        }
    }

    #[tokio::test]
    #[ignore = "Takes too long"]
    async fn test_uniswap_v3_base_pool_m15() {
        unsafe {
            std::env::set_var("UNISWAP_BATCH_SIZE", "1000");
        }

        let instrument = Instrument {
            asset_id: "base_test_pool".to_owned(),
            pair: "0x4200000000000000000000000000000000000006_base_0x88A43bbDF9D098eEC7bCEda4e2494615dfD9bB9C".to_owned(),
            connection: Connection::UniswapV3,
            market_type: MarketType::Spot,
            limit: Some(5),
            timeframe: Timeframe::M15,
        };

        match UniswapV3::get_candles(instrument.clone()).await {
            Ok(result) => examine_candles(&result, instrument),
            Err(err) => panic!("Failed to fetch candles: {err}"),
        }
    }
}
