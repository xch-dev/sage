use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{normalized_non_empty_string, normalized_optional_string};

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
pub struct SageAppAuthor {
    name: String,
    avatar: Option<String>,
}

impl SageAppAuthor {
    pub fn new(name: impl Into<String>, avatar: Option<impl Into<String>>) -> anyhow::Result<Self> {
        Ok(Self {
            name: normalized_non_empty_string(name, "author name")?,
            avatar: normalized_optional_string(avatar),
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn avatar(&self) -> Option<&str> {
        self.avatar.as_deref()
    }
}
