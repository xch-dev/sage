use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct Unit {
    pub ticker: String,
    pub decimals: u8,
}

impl Unit {
    pub fn cat(ticker: String) -> Self {
        Self {
            ticker,
            decimals: 3,
        }
    }
}

pub static XCH: Lazy<Unit> = Lazy::new(|| Unit {
    ticker: "XCH".to_string(),
    decimals: 12,
});

pub static TXCH: Lazy<Unit> = Lazy::new(|| Unit {
    ticker: "TXCH".to_string(),
    decimals: 12,
});

pub static MOJOS: Lazy<Unit> = Lazy::new(|| Unit {
    ticker: "Mojos".to_string(),
    decimals: 0,
});
