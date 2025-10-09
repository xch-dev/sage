use std::sync::LazyLock;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct Unit {
    pub ticker: String,
    pub precision: u8,
}

impl Unit {
    pub fn cat(ticker: String) -> Self {
        Self {
            ticker,
            precision: 3,
        }
    }
}

pub static XCH: LazyLock<Unit> = LazyLock::new(|| Unit {
    ticker: "XCH".to_string(),
    precision: 12,
});

pub static TXCH: LazyLock<Unit> = LazyLock::new(|| Unit {
    ticker: "TXCH".to_string(),
    precision: 12,
});

pub static MOJOS: LazyLock<Unit> = LazyLock::new(|| Unit {
    ticker: "Mojos".to_string(),
    precision: 0,
});
