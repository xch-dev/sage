use std::str::FromStr;

use num_bigint::BigInt;
use serde::{Deserialize, Serialize};

/// Pertains to [CHIP-0007](https://github.com/Chia-Network/chips/blob/main/CHIPs/chip-0007.md) off-chain metadata for NFTs.
/// The `data` field in the spec is ommitted as it's not useful for wallet implementations at this time.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Chip0007Metadata {
    pub format: String,
    pub name: String,
    pub description: String,
    pub minting_tool: Option<String>,
    pub sensitive_content: Option<SensitiveContent>,
    pub attributes: Option<Vec<NftAttribute>>,
    pub collection: Option<Collection>,
}

impl Chip0007Metadata {
    pub fn is_sensitive(&self) -> bool {
        match &self.sensitive_content {
            Some(SensitiveContent::Flag(flag)) => *flag,
            Some(SensitiveContent::Items(items)) => !items.is_empty(),
            Some(SensitiveContent::FlagString(flag)) => matches!(flag.as_str(), "true"),
            None => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SensitiveContent {
    Flag(bool),
    Items(Vec<String>),
    FlagString(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NftAttribute {
    pub trait_type: serde_json::Value,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AttributeValue {
    Int(i64),
    BigInt(BigInt),
    String(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Collection {
    pub id: String,
    pub name: String,
    pub attributes: Option<Vec<CollectionAttribute>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CollectionAttribute {
    #[serde(rename = "type")]
    pub kind: AttributeValue,
    pub value: AttributeValue,
}

impl Chip0007Metadata {
    pub fn parse(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(bytes)
    }
}

impl FromStr for Chip0007Metadata {
    type Err = serde_json::Error;

    fn from_str(json: &str) -> Result<Self, Self::Err> {
        Self::parse(json)
    }
}

impl AttributeValue {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            AttributeValue::String(value) => Some(value),
            AttributeValue::Int(..) | AttributeValue::BigInt(..) => None,
        }
    }
}
