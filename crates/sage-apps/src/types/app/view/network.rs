use serde::{Serialize};
use specta::Type;
use crate::types::SageNetworkWhitelistEntry;

#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct SageNetworkWhitelistEntryView {
    scheme: String,
    host: String,
}

impl From<&SageNetworkWhitelistEntry> for SageNetworkWhitelistEntryView {
    fn from(entry: &SageNetworkWhitelistEntry) -> Self {
        Self {
            scheme: entry.scheme().to_string(),
            host: entry.host().to_string(),
        }
    }
}
