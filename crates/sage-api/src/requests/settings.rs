use indexmap::IndexMap;
use sage_config::Network;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::PeerRecord;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
pub struct GetPeers {}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetPeersResponse {
    pub peers: Vec<PeerRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct RemovePeer {
    pub ip: String,
    pub ban: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
pub struct RemovePeerResponse {}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AddPeer {
    pub ip: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
pub struct AddPeerResponse {}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
pub struct SetDiscoverPeers {
    pub discover_peers: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
pub struct SetDiscoverPeersResponse {}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
pub struct SetTargetPeers {
    pub target_peers: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
pub struct SetTargetPeersResponse {}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SetNetworkId {
    pub network_id: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
pub struct SetNetworkIdResponse {}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
pub struct SetDeriveAutomatically {
    pub fingerprint: u32,
    pub derive_automatically: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
pub struct SetDeriveAutomaticallyResponse {}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
pub struct SetDerivationBatchSize {
    pub fingerprint: u32,
    pub derivation_batch_size: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
pub struct SetDerivationBatchSizeResponse {}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
pub struct GetNetworks {}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetNetworksResponse {
    pub networks: IndexMap<String, Network>,
}
