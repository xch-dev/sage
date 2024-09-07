use chia_wallet_sdk::{MAINNET_CONSTANTS, TESTNET11_CONSTANTS};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Network {
    pub default_port: u16,
    pub ticker: String,
    pub address_prefix: String,
    pub precision: u8,
    pub genesis_challenge: String,
    pub agg_sig_me: String,
    pub dns_introducers: Vec<String>,
}

pub static MAINNET: Lazy<Network> = Lazy::new(|| Network {
    default_port: 8444,
    ticker: "XCH".to_string(),
    address_prefix: "xch".to_string(),
    precision: 12,
    genesis_challenge: hex::encode(MAINNET_CONSTANTS.genesis_challenge),
    agg_sig_me: hex::encode(MAINNET_CONSTANTS.agg_sig_me_additional_data),
    dns_introducers: vec![
        "dns-introducer.chia.net".to_string(),
        "chia.ctrlaltdel.ch".to_string(),
        "seeder.dexie.space".to_string(),
        "chia.hoffmang.com".to_string(),
    ],
});

pub static TESTNET11: Lazy<Network> = Lazy::new(|| Network {
    default_port: 58444,
    ticker: "TXCH".to_string(),
    address_prefix: "txch".to_string(),
    precision: 12,
    genesis_challenge: hex::encode(TESTNET11_CONSTANTS.genesis_challenge),
    agg_sig_me: hex::encode(TESTNET11_CONSTANTS.agg_sig_me_additional_data),
    dns_introducers: vec!["dns-introducer-testnet11.chia.net".to_string()],
});
