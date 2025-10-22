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
    #[ignore = "Takes too much time"]
    async fn test_uniswap_v3_ethereum_usdc_weth_m15() {
        unsafe {
            std::env::set_var("UNISWAP_BATCH_SIZE", "1000");
            std::env::set_var("UNISWAP_MIN_CANDLES", "10");
            std::env::set_var("UNISWAP_RPC_DELAY_MS", "100");
        }

        let instrument = Instrument {
            asset_id: "ethereum_usdc_weth".to_owned(),
            pair: "ethereum_0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640".to_owned(),
            connection: Connection::UniswapV3,
            market_type: MarketType::Spot,
            timeframe: Timeframe::M15,
        };

        match UniswapV3::get_candles(instrument).await {
            Ok(result) => examine_candles(&result),
            Err(err) => panic!("Failed to fetch candles: {err}"),
        }
    }

    #[tokio::test]
    async fn test_uniswap_v3_base_pool_m15() {
        unsafe {
            std::env::set_var("UNISWAP_BATCH_SIZE", "1000");
            std::env::set_var("UNISWAP_MIN_CANDLES", "10");
            std::env::set_var("UNISWAP_RPC_DELAY_MS", "100");
        }

        let instrument = Instrument {
            asset_id: "base_test_pool".to_owned(),
            pair: "base_0xe1bed6aadba5471700f16a47eee2504346b724ad".to_owned(),
            connection: Connection::UniswapV3,
            market_type: MarketType::Spot,
            timeframe: Timeframe::M15,
        };

        match UniswapV3::get_candles(instrument).await {
            Ok(result) => examine_candles(&result),
            Err(err) => panic!("Failed to fetch candles: {err}"),
        }
    }

    #[tokio::test]
    #[ignore = "Takes too much time"]
    async fn test_uniswap_v3_ethereum_inverted_m15() {
        unsafe {
            std::env::set_var("UNISWAP_BATCH_SIZE", "1000");
            std::env::set_var("UNISWAP_MIN_CANDLES", "10");
            std::env::set_var("UNISWAP_RPC_DELAY_MS", "100");
        }

        let instrument = Instrument {
            asset_id: "ethereum_usdc_weth_inverted".to_owned(),
            pair: "ethereum_0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640_inverted".to_owned(),
            connection: Connection::UniswapV3,
            market_type: MarketType::Spot,
            timeframe: Timeframe::M15,
        };

        match UniswapV3::get_candles(instrument).await {
            Ok(result) => {
                examine_candles(&result);

                // Additional check for inverted price range
                if let Some(candle) = result.last() {
                    assert!(
                        candle.close > 1000.0 && candle.close < 10000.0,
                        "Inverted price should be in range $1000-$10000 per ETH, got: {}",
                        candle.close
                    );
                }
            }
            Err(err) => panic!("Failed to fetch candles: {err}"),
        }
    }
}
