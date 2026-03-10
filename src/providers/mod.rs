pub mod base;

// Crypto

pub mod binance {
    mod __test__;
    pub mod main;
}

pub mod okx {
    mod __test__;
    pub mod main;
}

pub mod bybit {
    mod __test__;
    pub mod main;
    mod types;
}

pub mod blofin {
    mod __test__;
    pub mod main;
}

pub mod bingx {
    mod __test__;
    pub mod main;
}

pub mod htx {
    mod __test__;
    pub mod main;
    mod types;
}

pub mod mexc {
    mod __test__;
    pub mod main;
    mod types;
}

pub mod coingecko {
    mod __test__;
    pub mod main;
    pub mod types;
}

pub mod hyperliquid {
    mod __test__;
    pub mod main;
    mod types;
}

pub mod freedx {
    mod __test__;
    pub mod main;
}

// Stocks

pub mod alpha_vantage {
    mod __test__;
    pub mod main;
    mod utils;
}
