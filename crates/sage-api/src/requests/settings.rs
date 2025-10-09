use sage_config::{Network, NetworkList};
use serde::{Deserialize, Serialize};

use crate::PeerRecord;

/// List all network peers
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Peers",
        description = "List all network peers the wallet is connected to."
    )
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetPeers {}

/// Response containing peer list
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "Peers"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetPeersResponse {
    /// List of connected peers
    pub peers: Vec<PeerRecord>,
}

/// Remove a peer from the connection list
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Peers",
        description = "Remove a specific peer from the connection list."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct RemovePeer {
    /// IP address or hostname of the peer
    #[cfg_attr(feature = "openapi", schema(example = "127.0.0.1:8444"))]
    pub ip: String,
    /// Whether to ban the peer from reconnecting
    #[cfg_attr(feature = "openapi", schema(example = false))]
    pub ban: bool,
}

/// Add a new peer to connect to
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Peers",
        description = "Add a new peer to connect to by host and port."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AddPeer {
    /// IP address or hostname with port
    #[cfg_attr(feature = "openapi", schema(example = "node.example.com:8444"))]
    pub ip: String,
}

/// Enable or disable automatic peer discovery
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Peers",
        description = "Enable or disable automatic peer discovery via DNS."
    )
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SetDiscoverPeers {
    /// Whether to enable peer discovery
    #[cfg_attr(feature = "openapi", schema(example = true))]
    pub discover_peers: bool,
}

/// Set target number of peers to maintain
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Peers",
        description = "Set the target number of peers to maintain connections with."
    )
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SetTargetPeers {
    /// Target number of peer connections
    #[cfg_attr(feature = "openapi", schema(example = 8))]
    pub target_peers: u32,
}

/// Set the active network
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Network Settings",
        description = "Set the active network (mainnet, testnet, etc.)."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SetNetwork {
    /// Network name to switch to
    #[cfg_attr(feature = "openapi", schema(example = "mainnet"))]
    pub name: String,
}

/// Override network settings for a specific wallet
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Network Settings",
        description = "Override network settings with custom values."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SetNetworkOverride {
    /// Wallet fingerprint to override network for
    #[cfg_attr(feature = "openapi", schema(example = 1_234_567_890))]
    pub fingerprint: u32,
    /// Network name (null to reset to default)
    #[cfg_attr(feature = "openapi", schema(example = "testnet11"))]
    pub name: Option<String>,
}

/// List available networks
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Network Settings",
        description = "List available networks (mainnet, testnet, etc.)."
    )
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetNetworks {}

pub type GetNetworksResponse = NetworkList;

/// Get current network information
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Network Settings",
        description = "Get current network configuration and status."
    )
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetNetwork {}

/// Response containing current network information
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "Network Settings"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
pub struct GetNetworkResponse {
    /// Current network configuration
    pub network: Network,
    /// Network type classification
    pub kind: NetworkKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum NetworkKind {
    Mainnet,
    Testnet,
    Unknown,
}

/// Enable or disable delta sync
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Network Settings",
        description = "Enable or disable delta sync mode for faster synchronization."
    )
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SetDeltaSync {
    /// Whether to enable delta sync
    #[cfg_attr(feature = "openapi", schema(example = true))]
    pub delta_sync: bool,
}

/// Override delta sync settings for a specific wallet
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Network Settings",
        description = "Override delta sync settings with custom configuration."
    )
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SetDeltaSyncOverride {
    /// Wallet fingerprint
    #[cfg_attr(feature = "openapi", schema(example = 1_234_567_890))]
    pub fingerprint: u32,
    /// Delta sync setting (null to use default)
    pub delta_sync: Option<bool>,
}

/// Set the change address for transactions
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Addresses",
        description = "Set a custom change address for transaction outputs."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SetChangeAddress {
    /// Wallet fingerprint
    #[cfg_attr(feature = "openapi", schema(example = 1_234_567_890))]
    pub fingerprint: u32,
    /// Change address (null to use default derivation)
    #[cfg_attr(feature = "openapi", schema(example = "xch1..."))]
    pub change_address: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
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
