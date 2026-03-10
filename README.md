# candles-rs

A Rust library for fetching candlestick (OHLCV) data from multiple cryptocurrency exchanges, DEX aggregators, and stock markets with automatic pagination. Built by [Flicker](https://flicker.finance).

## Features

- **Multi-Exchange Support**:
  - **CEX**: Binance, OKX, Bybit, BloFin, BingX, HTX, MEXC, Hyperliquid, FreeDX
  - **DEX Aggregators**: CoinGecko (GeckoTerminal)
  - **Stocks**: AlphaVantage
- **Automatic Pagination**: Request any number of candles (e.g., 2000+) and the library handles batching
- **Time Range Queries**: Fetch candles by `start_time`, `end_time`, or both
- **Multiple Timeframes**: 3m, 5m, 15m, 30m, 1h, 4h, 1d, 1w, 1M
- **Async/Await**: Built with async Rust
- **Type Safety**: Strongly typed with comprehensive error handling

## Installation

```toml
[dependencies]
candles-rs = "0.1.10"
```

## Quick Start

```rust
use candles_rs::{
    connections::Connection,
    types::{Instrument, MarketType, Timeframe},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let instrument = Instrument {
        asset_id: "bitcoin".to_string(),
        pair: "BTCUSDT".to_string(),
        asset_symbol: "BTC".to_string(),
        connection: Connection::Binance,
        market_type: MarketType::Spot,
        timeframe: Timeframe::H1,
        limit: None,
        start_time: None,
        end_time: None,
    };

    let candles = instrument.connection.get_candles(instrument).await?;

    for candle in candles {
        println!(
            "Time: {}, O: {}, H: {}, L: {}, C: {}, V: {}",
            candle.timestamp, candle.open, candle.high,
            candle.low, candle.close, candle.volume
        );
    }

    Ok(())
}
```

## Pagination

Request more candles than a single API call allows. The library automatically paginates backwards:

```rust
let instrument = Instrument {
    asset_id: "bitcoin".to_string(),
    pair: "BTCUSDT".to_string(),
    asset_symbol: "BTC".to_string(),
    connection: Connection::Binance,
    market_type: MarketType::Spot,
    timeframe: Timeframe::H1,
    limit: Some(2000),
    start_time: None,
    end_time: None,
};

// Returns 2000 H1 candles in ascending order
let candles = instrument.connection.get_candles(instrument).await?;
```

You can also specify a time range:

```rust
let instrument = Instrument {
    // ...
    limit: None,
    start_time: Some(1704067200000), // epoch ms
    end_time: Some(1706745600000),
    // ...
};
```

## Supported Exchanges

| Exchange | Spot | Derivatives | Max per batch |
|----------|------|-------------|---------------|
| Binance | Y | Y | 1000 |
| OKX | Y | Y | 100 (auto-switches to /history-candles for older data) |
| Bybit | Y | Y | 1000 |
| BloFin | Y | Y | 100 |
| BingX | Y | Y | 1000 |
| HTX | Y | Y | 2000 (Spot has no time-based pagination) |
| MEXC | Y | Y | 500 |
| Hyperliquid | - | Y | 5000 |
| FreeDX | Y | Y | 300 |
| CoinGecko | Y | - | 1000 |
| AlphaVantage | Y | - | Full dataset |

### CoinGecko (GeckoTerminal)

Pair format: `tokenAddress_chain_poolAddress`

```rust
let instrument = Instrument {
    asset_id: "ethereum_usdc_weth".to_string(),
    pair: "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2_eth_0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640".to_string(),
    asset_symbol: "WETH".to_string(),
    connection: Connection::CoinGecko,
    market_type: MarketType::Spot,
    timeframe: Timeframe::H1,
    limit: None,
    start_time: None,
    end_time: None,
};
```

Supported chains: Ethereum, Base, BNB Chain, Solana

### AlphaVantage

Requires `ALPHA_VANTAGE_API_KEY` environment variable.

```rust
let instrument = Instrument {
    asset_id: "NVDA".to_string(),
    pair: "NVDA".to_string(),
    asset_symbol: "NVDA".to_string(),
    connection: Connection::AlphaVantage,
    market_type: MarketType::Spot,
    timeframe: Timeframe::D1,
    limit: None,
    start_time: None,
    end_time: None,
};
```

### FreeDX

Spot symbol: `BTC-USDT`, Futures symbol: `BTC-PERP`. Supported timeframes: 5m, 15m, 30m, 1h, 4h. Volume is automatically converted from quote to base asset.

### Hyperliquid

Uses single coin symbols (e.g., "BTC", "ETH", "SOL") rather than trading pairs.

## Data Types

```rust
pub struct Instrument {
    pub asset_id: String,
    pub asset_symbol: String,
    pub pair: String,
    pub limit: Option<u64>,       // Number of candles to fetch
    pub start_time: Option<i64>,  // Epoch milliseconds
    pub end_time: Option<i64>,    // Epoch milliseconds
    pub connection: Connection,
    pub market_type: MarketType,
    pub timeframe: Timeframe,
}

pub struct Candle {
    pub timestamp: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}
```

## License

This project is open source and available under the [MIT License](LICENSE).

## About Flicker Finance

Developed and maintained by [Flicker Finance](https://flicker.finance).
