#[cfg(test)]
mod test {
    use crate::{
        base::BaseConnection,
        connections::Connection,
        types::{Instrument, MarketType, Timeframe},
        uniswap_v3::main::UniswapV3,
    };

    #[tokio::test]
    async fn test_uniswap_v3_ethereum_usdc_weth_m15() {
        let instrument = Instrument {
            asset_id: "ethereum_usdc_weth".to_owned(),
            pair: "ethereum_0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640".to_owned(),
            connection: Connection::UniswapV3,
            market_type: MarketType::Spot,
            timeframe: Timeframe::M15,
        };

        match UniswapV3::get_candles(instrument).await {
            Ok(result) => {
                println!("Fetched {} candles", result.len());
                if !result.is_empty() {
                    println!("Latest candle: {:?}", result.first());
                    println!("Oldest candle: {:?}", result.last());
                }
                assert!(!result.is_empty(), "Should fetch at least one candle");
            }
            Err(err) => panic!("Failed to fetch candles: {err}"),
        }
    }

    #[tokio::test]
    async fn test_uniswap_v3_base_pool_m15() {
        let instrument = Instrument {
            asset_id: "base_weth_usdc".to_owned(),
            pair: "base_0xd0b53d9277642d899df5c87a3966a349a798f224".to_owned(),
            connection: Connection::UniswapV3,
            market_type: MarketType::Spot,
            timeframe: Timeframe::M15,
        };

        match UniswapV3::get_candles(instrument).await {
            Ok(result) => {
                println!("Fetched {} candles", result.len());
                if !result.is_empty() {
                    println!("Latest candle: {:?}", result.last());
                    println!("Oldest candle: {:?}", result.first());
                }
                assert!(!result.is_empty(), "Should fetch at least one candle");
            }
            Err(err) => panic!("Failed to fetch candles: {err}"),
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_uniswap_v3_ethereum_inverted_m15() {
        let instrument = Instrument {
            asset_id: "ethereum_usdc_weth_inverted".to_owned(),
            pair: "ethereum_0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640_inverted".to_owned(),
            connection: Connection::UniswapV3,
            market_type: MarketType::Spot,
            timeframe: Timeframe::M15,
        };

        match UniswapV3::get_candles(instrument).await {
            Ok(result) => {
                println!("Fetched {} candles", result.len());
                if !result.is_empty() {
                    println!("Latest candle: {:?}", result.last());
                    println!("Oldest candle: {:?}", result.first());

                    if let Some(candle) = result.last() {
                        assert!(
                            candle.close > 1000.0 && candle.close < 10000.0,
                            "Inverted price should be in range $1000-$10000 per ETH, got: {}",
                            candle.close
                        );
                    }
                }
                assert!(!result.is_empty(), "Should fetch at least one candle");
            }
            Err(err) => panic!("Failed to fetch candles: {err}"),
        }
    }
}
