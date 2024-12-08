use std::fmt;

use serde::{Deserialize, Serialize};
use specta::Type;

pub const MAX_JS_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(untagged)]
pub enum Amount {
    String(String),
    Number(u64),
}

impl Amount {
    pub fn u64(value: u64) -> Self {
        if value > MAX_JS_SAFE_INTEGER {
            Self::String(value.to_string())
        } else {
            Self::Number(value)
        }
    }

    pub fn u128(value: u128) -> Self {
        if value > u128::from(MAX_JS_SAFE_INTEGER) {
            Self::String(value.to_string())
        } else {
            Self::Number(value as u64)
        }
    }

    pub fn to_u64(&self) -> Option<u64> {
        match self {
            Self::String(value) => value.parse().ok(),
            Self::Number(value) => Some(*value),
        }
    }

    pub fn to_u16(&self) -> Option<u16> {
        match self {
            Self::String(value) => value.parse().ok(),
            Self::Number(value) => (*value).try_into().ok(),
        }
    }

    pub fn to_u128(&self) -> Option<u128> {
        match self {
            Self::String(value) => value.parse().ok(),
            Self::Number(value) => Some(*value as u128),
        }
    }
}

impl fmt::Display for Amount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::String(value) => write!(f, "{value}"),
            Self::Number(value) => write!(f, "{value}"),
        }
    }
}
