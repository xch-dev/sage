use sage_config::{Network, NetworkList};
use serde::{Deserialize, Serialize};

use crate::PeerRecord;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct GetPeers {}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct GetPeersResponse {
    pub peers: Vec<PeerRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct RemovePeer {
    pub ip: String,
    pub ban: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct AddPeer {
    pub ip: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct SetDiscoverPeers {
    pub discover_peers: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct SetTargetPeers {
    pub target_peers: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct SetNetwork {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct SetNetworkOverride {
    pub fingerprint: u32,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct GetNetworks {}

pub type GetNetworksResponse = NetworkList;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct GetNetwork {}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct GetNetworkResponse {
    pub network: Network,
    pub kind: NetworkKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[serde(rename_all = "snake_case")]
pub enum NetworkKind {
    Mainnet,
    Testnet,
    Unknown,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct SetDeltaSync {
    pub delta_sync: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct SetDeltaSyncOverride {
    pub fingerprint: u32,
    pub delta_sync: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct SetChangeAddress {
    pub fingerprint: u32,
    pub change_address: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct EmptyResponse {}

pub type AddPeerResponse = EmptyResponse;
pub type RemovePeerResponse = EmptyResponse;
pub type SetDiscoverPeersResponse = EmptyResponse;
pub type SetTargetPeersResponse = EmptyResponse;
pub type SetNetworkResponse = EmptyResponse;
pub type SetNetworkOverrideResponse = EmptyResponse;
pub type SetDeltaSyncResponse = EmptyResponse;
pub type SetDeltaSyncOverrideResponse = EmptyResponse;
pub type SetChangeAddressResponse = EmptyResponse;
