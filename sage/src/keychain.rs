use serde::{Deserialize, Serialize};
use serde_with::{serde_as, Bytes};

#[serde_as]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
pub enum Key {
    PublicKey(#[serde_as(as = "Bytes")] [u8; 48]),
}
