# candles-rs

A Rust library for fetching candlestick (OHLCV) data from multiple cryptocurrency exchanges and decentralized exchanges. Built by [Flicker](https://flicker.finance), this library provides a unified interface to access market data from centralized and decentralized sources.

## Features

- **Multi-Exchange Support**: Fetch candlestick data from:
  - **Centralized Exchanges (CEX)**:
    - Binance (Spot & Derivatives)
    - OKX
    - Bybit
    - BloFin
    - BingX
    - HTX (Huobi)
    - MEXC (Spot & Derivatives)
  - **Decentralized Exchanges (DEX)**:
    - Uniswap V3
      - Supports: Ethereum, Polygon, Arbitrum, Optimism, Base, BNB Chain, Celo, Avalanche
      - Aggregates on-chain swap data into OHLCV candles
      - Customizable price inversion for human-readable prices
  - **Data Aggregators**:
    - Coingecko Web3 Data API
      - Supports all major EVM chains
      - Pre-aggregated OHLCV data for DEX pairs
      - Fast and reliable via Coingecko infrastructure
- **Unified Interface**: Common API across all exchanges
- **Multiple Timeframes**: Support for 3m, 5m, 15m, 30m, 1h, 4h, 1d, 1w, 1M intervals
- **Async/Await**: Built with async Rust for efficient data fetching
- **Type Safety**: Strongly typed with comprehensive error handling

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
candles-rs = "0.1.0"
```

## Quick Start

```rust
use candles_rs::{
    connections::Connection,
    types::{Instrument, MarketType, Timeframe},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create an instrument configuration
    let instrument = Instrument {
        asset_id: "bitcoin".to_string(),
        pair: "BTCUSDT".to_string(),
        connection: Connection::Binance,
        market_type: MarketType::Spot,
        timeframe: Timeframe::H1,
    };

    // Fetch candlestick data
    let candles = instrument.connection.get_candles(instrument).await?;

    for candle in candles {
        println!(
            "Time: {}, Open: {}, High: {}, Low: {}, Close: {}, Volume: {}",
            candle.timestamp, candle.open, candle.high,
            candle.low, candle.close, candle.volume
        );
    }

    Ok(())
}
```

## Supported Exchanges

### Centralized Exchanges (CEX)

#### Binance
- **Spot Markets**: `https://www.binance.com/api/v3/klines`
- **Derivatives**: `https://fapi.binance.com/fapi/v1/klines`

#### OKX
- **All Markets**: `https://www.okx.com/api/v5/market/candles`

#### Bybit
- **All Markets**: Exchange-specific implementation

#### BloFin
- **All Markets**: Exchange-specific implementation

#### BingX
- **All Markets**: Exchange-specific implementation

#### HTX (Huobi)
- **All Markets**: Exchange-specific implementation

#### MEXC
- **Spot & Derivatives**: Exchange-specific implementation

### Decentralized Exchanges (DEX)

#### Uniswap V3
Fetches on-chain swap data using direct RPC calls via Alloy.

**Supported Chains**: Ethereum, Base, BNB Chain

**Configuration**:
```bash
# Global RPC override (applies to all chains)
export RPC_URL="https://your-rpc-endpoint.com"

# Or per-chain RPC URLs
export ETHEREUM_RPC_URL="https://eth-mainnet.g.alchemy.com/v2/YOUR-API-KEY"
export BASE_RPC_URL="https://base-mainnet.g.alchemy.com/v2/YOUR-API-KEY"
export BNB_RPC_URL="https://bsc-dataseed1.binance.org"

# Optional: Adjust batch size for eth_getLogs (default: 1000)
# Some RPCs have stricter limits (e.g., 500 for some Polygon nodes)
export UNISWAP_BATCH_SIZE=500

# Optional: Set minimum number of candles required (default: 250)
export UNISWAP_MIN_CANDLES=300

# Optional: Set delay between RPC calls in milliseconds (default: 50)
# Increase this if you're hitting rate limits (e.g., 200ms = 5 calls/second)
export UNISWAP_RPC_DELAY_MS=100
```

**Usage**:
```rust
use candles_rs::{connections::Connection, types::*};

let instrument = Instrument {
    asset_id: "ethereum_usdc_weth".to_string(),
    pair: "ethereum_0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640_inverted".to_string(),
    connection: Connection::UniswapV3,
    market_type: MarketType::Spot,
    timeframe: Timeframe::M15,
};

let candles = instrument.connection.get_candles(instrument).await?;
```

**Pair Format**: `chain_poolAddress` or `chain_poolAddress_inverted`
- Use `_inverted` suffix for human-readable prices (e.g., $4,021 vs 0.000249)
- **Important**: Must be a Uniswap V3 **pool** address, not a router contract
- Find pool addresses at [Uniswap Info](https://info.uniswap.org)

**Usage**:
```rust
use candles_rs::{connections::Connection, types::*};

let instrument = Instrument {
    asset_id: "ethereum_usdc_weth".to_string(),
    pair: "eth_0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640".to_string(),
    connection: Connection::Moralis,
    market_type: MarketType::Spot,
    timeframe: Timeframe::M15,
};

let candles = instrument.connection.get_candles(instrument).await?;
```
## Data Types

### Timeframe
```rust
pub enum Timeframe {
    M3,   // 3 minutes
    M5,   // 5 minutes
    M15,  // 15 minutes
    M30,  // 30 minutes
    H1,   // 1 hour
    H4,   // 4 hours
    D1,   // 1 day
    W1,   // 1 week
    MN1,  // 1 month
}
```

### MarketType
```rust
pub enum MarketType {
    Spot,        // Spot trading
    Derivatives, // Futures/derivatives
}
```

### Candle
```rust
pub struct Candle {
    pub timestamp: i64,  // Unix timestamp
    pub open: f64,       // Opening price
    pub high: f64,       // Highest price
    pub low: f64,        // Lowest price
    pub close: f64,      // Closing price
    pub volume: f64,     // Volume in base asset
}
```

## Error Handling

The library uses a comprehensive error system:

```rust
pub enum CandlesError {
    ConnectionNotFound(String),  // Invalid exchange connection
    ApiError(String),           // API request failures
    Reqwest(reqwest::Error),    // HTTP client errors
    Other(String),              // General errors
}
```

## Examples

### Fetching from Different Exchanges

```rust
use candles_rs::{connections::Connection, types::*};

// Fetch from OKX
let okx_instrument = Instrument {
    asset_id: "BTC-USDT".to_string(),
    pair: "BTC-USDT".to_string(),
    connection: Connection::OKX,
    market_type: MarketType::Spot,
    timeframe: Timeframe::H4,
};

let candles = okx_instrument.connection.get_candles(okx_instrument).await?;
```

### Multiple Timeframes

```rust
let timeframes = vec![
    Timeframe::M15,
    Timeframe::H1,
    Timeframe::D1,
];

for timeframe in timeframes {
    let instrument = Instrument {
        asset_id: "ETHUSDT".to_string(),
        pair: "ETHUSDT".to_string(),
        connection: Connection::Binance,
        market_type: MarketType::Spot,
        timeframe,
    };

    let candles = instrument.connection.get_candles(instrument).await?;
    println!("Fetched {} candles for {:?}", candles.len(), timeframe);
}
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is open source and available under the [MIT License](LICENSE).

## About Flicker Finance

This library is developed and maintained by [Flicker Finance](https://flicker.finance), a platform for cryptocurrency trading and market analysis.

---

**Note**: This library is for educational and development purposes. Always ensure you comply with each exchange's terms of service and rate limits when using their APIs.
