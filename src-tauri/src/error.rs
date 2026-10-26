use std::fmt;

use sage_api::{ErrorKind, RecipientError};
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Error {
    pub kind: ErrorKind,
    pub reason: String,
    /// The recipients which couldn't be used, if that's why the request failed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[specta(optional)]
    pub recipients: Option<Vec<RecipientError>>,
}

impl From<sage::Error> for Error {
    fn from(error: sage::Error) -> Self {
        Self {
            kind: error.kind(),
            reason: error.to_string(),
            recipients: error.recipient_errors().map(<[RecipientError]>::to_vec),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.reason)
    }
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        Self {
            kind: ErrorKind::Internal,
            reason: error.to_string(),
            recipients: None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self {
            kind: ErrorKind::Internal,
            reason: error.to_string(),
            recipients: None,
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
