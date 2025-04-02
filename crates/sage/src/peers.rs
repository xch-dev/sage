use std::{
    collections::{HashMap, HashSet},
    net::IpAddr,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Peers {
    pub connections: HashSet<IpAddr>,
    pub user_managed: HashSet<IpAddr>,
    pub banned: HashMap<IpAddr, u64>,
}

impl Peers {
    pub fn to_bytes(&self) -> bincode::Result<Vec<u8>> {
        bincode::serialize(self)
    }

    pub fn from_bytes(bytes: &[u8]) -> bincode::Result<Self> {
        bincode::deserialize(bytes)
    }
}
