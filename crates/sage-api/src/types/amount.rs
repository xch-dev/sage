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

    pub fn to_mojos(&self, decimals: u8) -> Option<u128> {
        let mojos = &self.0 * 10u64.pow(decimals.into());
        mojos.to_u128()
    }
}
