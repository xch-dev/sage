use hex_literal::hex;
use serde::{Deserialize, Serialize};
use serde_with::{hex::Hex, serde_as};

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Network {
    pub default_port: u16,
    #[serde_as(as = "Hex")]
    pub genesis_challenge: [u8; 32],
    pub agg_sig_me: Option<[u8; 32]>,
    pub dns_introducers: Vec<String>,
}

impl Network {
    pub fn default_mainnet() -> Self {
        Self {
            default_port: 8444,
            genesis_challenge: hex!(
                "ccd5bb71183532bff220ba46c268991a3ff07eb358e8255a65c30a2dce0e5fbb"
            ),
            agg_sig_me: None,
            dns_introducers: vec![
                "dns-introducer.chia.net".to_string(),
                "chia.ctrlaltdel.ch".to_string(),
                "seeder.dexie.space".to_string(),
                "chia.hoffmang.com".to_string(),
            ],
        }
    }

    pub fn default_testnet11() -> Self {
        Self {
            default_port: 58444,
            genesis_challenge: hex!(
                "37a90eb5185a9c4439a91ddc98bbadce7b4feba060d50116a067de66bf236615"
            ),
            agg_sig_me: None,
            dns_introducers: vec!["dns-introducer-testnet11.chia.net".to_string()],
        }
    }
}
