use serde::{Deserialize, Serialize};
use specta::Type;

use crate::types::normalized_non_empty_string;

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
pub struct SageAppDonation {
    address: String,
}

impl SageAppDonation {
    pub fn new(address: impl Into<String>) -> anyhow::Result<Self> {
        let address = normalized_non_empty_string(address, "donation address")?;

        if !address.starts_with("xch") && !address.starts_with("txch") {
            anyhow::bail!("invalid donation address format");
        }

        Ok(Self { address })
    }

    pub fn address(&self) -> &str {
        &self.address
    }
}
