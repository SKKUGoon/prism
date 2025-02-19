use std::env;

#[allow(dead_code)] // There can be unused variable
pub struct PrismEnvConfig {
    // Symbols
    pub symbol_binance_fut: String,
    pub symbol_binance_spt: String,
    pub symbol_upbit_krw: String,
    pub symbol_upbit_btc: String,
    pub symbol_upbit_usdt: String,

    // Database Tables
    pub table_fut: String, // For Raw data
    pub table_spt: String,
    pub table_strat1: String, // For Strategy 1
    // Other
    pub data_dump: bool,
    pub channel_capacity: usize,
}

pub fn read_env_config() -> PrismEnvConfig {
    PrismEnvConfig {
        // Symbols
        symbol_binance_fut: env::var("SYMBOLS_BINANCE_FUT")
            .unwrap_or_else(|_| "NO_SYMBOL".to_string()),
        symbol_binance_spt: env::var("SYMBOLS_BINANCE_SPT")
            .unwrap_or_else(|_| "NO_SYMBOL".to_string()),
        symbol_upbit_krw: env::var("SYMBOLS_UPBIT_KRW").unwrap_or_else(|_| "NO_SYMBOL".to_string()),
        symbol_upbit_btc: env::var("SYMBOLS_UPBIT_BTC").unwrap_or_else(|_| "NO_SYMBOL".to_string()),
        symbol_upbit_usdt: env::var("SYMBOLS_UPBIT_USDT")
            .unwrap_or_else(|_| "NO_SYMBOL".to_string()),

        // Database Tables
        table_fut: env::var("TABLE_FUT").unwrap_or_else(|_| "unspecified".to_string()),
        table_spt: env::var("TABLE_SPT").unwrap_or_else(|_| "unspecified".to_string()),
        table_strat1: env::var("STRATEGY1_TABLE").unwrap_or_else(|_| "unspecified".to_string()),

        // Other
        data_dump: env::var("DATA_DUMP").unwrap_or_else(|_| "false".to_string()) == "true",
        channel_capacity: env::var("CHANNEL_CAPACITY")
            .unwrap_or_else(|_| "999".to_string())
            .parse()
            .unwrap_or(999),
    }
}
