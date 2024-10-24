use std::fmt;

use bigdecimal::BigDecimal;
use num_traits::ToPrimitive;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct Amount(BigDecimal);

impl Amount {
    pub fn new(value: BigDecimal) -> Self {
        Self(value)
    }

    pub fn from_mojos(mojos: u128, decimals: u8) -> Self {
        Self(BigDecimal::from(mojos) / 10u64.pow(decimals.into()))
    }

    pub fn to_mojos_u128(&self, decimals: u8) -> Option<u128> {
        let mojos = &self.0 * 10u64.pow(decimals.into());

        if mojos.normalized().fractional_digit_count() > 0 {
            return None;
        }

        mojos.to_u128()
    }

    pub fn to_mojos(&self, decimals: u8) -> Option<u64> {
        let mojos = &self.0 * 10u64.pow(decimals.into());

        if mojos.normalized().fractional_digit_count() > 0 {
            return None;
        }

        mojos.to_u64()
    }

    pub fn to_ten_thousandths(&self) -> Option<u16> {
        let mojos = &self.0 * 100u16;

        if mojos.normalized().fractional_digit_count() > 0 {
            return None;
        }

        mojos.to_u16()
    }
}

impl fmt::Display for Amount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
