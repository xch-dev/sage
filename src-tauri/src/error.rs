use std::fmt;

use sage_api::{ErrorKind, HandleErrorCode};
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Error {
    pub kind: ErrorKind,
    pub reason: String,
    /// Why an XCHandles request failed, if it did
    pub handle_error_code: Option<HandleErrorCode>,
}

impl From<sage::Error> for Error {
    fn from(error: sage::Error) -> Self {
        Self {
            kind: error.kind(),
            reason: error.to_string(),
            handle_error_code: error.handle_error_code(),
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
            handle_error_code: None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self {
            kind: ErrorKind::Internal,
            reason: error.to_string(),
            handle_error_code: None,
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
