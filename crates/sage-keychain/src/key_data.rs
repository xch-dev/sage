use serde::{Deserialize, Serialize};
use serde_with::{Bytes, serde_as};

use crate::encrypt::Encrypted;

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[repr(u8)]
pub enum KeyData {
    Public {
        #[serde_as(as = "Bytes")]
        master_pk: [u8; 48],
    },
    Secret {
        #[serde_as(as = "Bytes")]
        master_pk: [u8; 48],
        entropy: bool,
        encrypted: Encrypted,
    },
    Vault {
        #[serde_as(as = "Bytes")]
        launcher_id: [u8; 32],
    },
    Watch {
        #[serde_as(as = "Vec<Bytes>")]
        p2_puzzle_hashes: Vec<[u8; 32]>,
    },
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretKeyData(#[serde_as(as = "Bytes")] pub Vec<u8>);
