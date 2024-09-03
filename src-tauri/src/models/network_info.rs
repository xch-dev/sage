use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct NetworkInfo {
    pub default_port: u16,
    pub genesis_challenge: String,
    pub agg_sig_me: Option<String>,
    pub dns_introducers: Vec<String>,
}
