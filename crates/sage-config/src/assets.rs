use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Default, Clone, Serialize, Deserialize, Type)]
pub struct Assets {
    pub tokens: IndexMap<String, Token>,
    pub profiles: IndexMap<String, Profile>,
    pub nfts: IndexMap<String, Nft>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Token {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ticker: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(default = "default_precision", skip_serializing_if = "is_3")]
    pub precision: u8,

    #[serde(default, skip_serializing_if = "is_false")]
    pub hidden: bool,
}

impl Default for Token {
    fn default() -> Self {
        Self {
            name: None,
            icon_url: None,
            ticker: None,
            description: None,
            precision: default_precision(),
            hidden: false,
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Type)]
pub struct Profile {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    #[serde(default, skip_serializing_if = "is_false")]
    pub hidden: bool,
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, Type)]
pub struct Nft {
    #[serde(default, skip_serializing_if = "is_false")]
    pub hidden: bool,
}

fn default_precision() -> u8 {
    3
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_3(precision: &u8) -> bool {
    *precision == 3
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_false(b: &bool) -> bool {
    !*b
}
