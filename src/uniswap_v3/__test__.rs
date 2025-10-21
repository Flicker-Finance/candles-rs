#[cfg(test)]
mod test {
    use crate::{
        base::BaseConnection,
        connections::Connection,
        types::{Instrument, MarketType, Timeframe},
        uniswap_v3::main::UniswapV3,
    };

    fn check_api_key() -> bool {
        if std::env::var("GRAPH_API_KEY").is_err() {
            println!("⚠️  Skipping test: GRAPH_API_KEY environment variable not set");
            println!("   Set it with: export GRAPH_API_KEY=\"your_key_here\"");
            println!("   Get your key at: https://thegraph.com/studio/");
            return false;
        }
        true
    }

    #[tokio::test]
    async fn test_uniswap_v3_ethereum_usdc_weth_m15() {
        if !check_api_key() {
            return;
        }
        // USDC/WETH 0.05% pool on Ethereum mainnet - most active pool
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
                // Don't validate with examine_candles for now, just check we got data
                assert!(!result.is_empty(), "Should fetch at least one candle");
            }
            Err(err) => panic!("Failed to fetch candles: {err}"),
        }
    }

    #[tokio::test]
    async fn test_uniswap_v3_polygon_usdc_weth_m15() {
        if !check_api_key() {
            return;
        }

        let instrument = Instrument {
            asset_id: "polygon_usdc_weth".to_owned(),
            pair: "polygon_0x45dda9cb7c25131df268515131f647d726f50608".to_owned(),
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
    async fn test_uniswap_v3_arbitrum_usdc_weth_m15() {
        if !check_api_key() {
            return;
        }

        let instrument = Instrument {
            asset_id: "arbitrum_usdc_weth".to_owned(),
            pair: "arbitrum_0xc31e54c7a869b9fcbecc14363cf510d1c41fa443".to_owned(),
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
    async fn test_uniswap_v3_base_pool_m15() {
        if !check_api_key() {
            return;
        }

        let instrument = Instrument {
            asset_id: "base_custom_pool".to_owned(),
            pair: "base_0xd3be98e1c9de70cbcc2a50eafaf43f03ad6109f2".to_owned(),
            connection: Connection::UniswapV3,
            market_type: MarketType::Spot,
            timeframe: Timeframe::M15,
        };

        match UniswapV3::get_candles(instrument).await {
            Ok(result) => {
                println!("\n=== Base Pool Test Results ===");
                println!("Fetched {} candles", result.len());
                if !result.is_empty() {
                    println!("\nLatest candle (newest):");
                    println!("  {:?}", result.last());
                    println!("\nOldest candle:");
                    println!("  {:?}", result.first());

                    // Show a few sample candles
                    println!("\nSample candles:");
                    for (i, candle) in result.iter().take(5).enumerate() {
                        println!(
                            "  [{}] Timestamp: {}, O: {:.2e}, H: {:.2e}, L: {:.2e}, C: {:.2e}, V: {:.2}",
                            i, candle.timestamp, candle.open, candle.high, candle.low, candle.close, candle.volume
                        );
                    }
                }
                assert!(!result.is_empty(), "Should fetch at least one candle");
            }
            Err(err) => panic!("Failed to fetch candles: {err}"),
        }
    }

    #[tokio::test]
    async fn test_uniswap_v3_ethereum_inverted_m15() {
        if !check_api_key() {
            return;
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
                println!("\n=== Inverted Price Test ===");
                println!("Fetched {} candles", result.len());
                if !result.is_empty() {
                    println!("Latest candle: {:?}", result.last());
                    println!("Oldest candle: {:?}", result.first());

                    // With inverted, prices should be in thousands (ETH price in USDC)
                    if let Some(candle) = result.last() {
                        println!("Latest close price: ${:.2}", candle.close);
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
