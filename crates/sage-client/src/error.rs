use std::{io, net::AddrParseError};

use reqwest::StatusCode;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SageRpcError {
    #[error("Reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Config error: {0}")]
    Config(#[from] toml::de::Error),

    #[error("API error {0}: {1}")]
    Api(StatusCode, String),

    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Invalid path")]
    InvalidPath,

    #[error("Address parse error: {0}")]
    AddrParse(#[from] AddrParseError),

    #[error("Missing data directory")]
    MissingDataDir,
}
