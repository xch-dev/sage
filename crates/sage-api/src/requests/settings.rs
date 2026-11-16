use sage_config::{Network, NetworkList};
use serde::{Deserialize, Serialize};

use crate::PeerRecord;

/// List all network peers
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Peers",
        description = "List the peers Sage is connected to, with their peak heights."
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
        description = "Disconnect from a peer by IP address, optionally banning it for an hour.",
        response_type = "EmptyResponse"
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct RemovePeer {
    /// IP address of the peer
    #[cfg_attr(feature = "openapi", schema(example = "127.0.0.1"))]
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
        description = "Connect to a peer by IP address, on the network's default port.",
        response_type = "EmptyResponse"
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AddPeer {
    /// IP address of the peer, which is connected to on the network's default port
    #[cfg_attr(feature = "openapi", schema(example = "203.0.113.5"))]
    pub ip: String,
}

/// Enable or disable automatic peer discovery
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Peers",
        description = "Enable or disable connecting to peers found automatically, rather than only peers that were added.",
        response_type = "EmptyResponse"
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

/// Enable or disable looking up announced mempool transactions with Coinset
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Network",
        description = "Enable or disable looking up the wallet's mempool transactions with the Coinset API, so they show up before they're confirmed.",
        response_type = "EmptyResponse"
    )
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SetMempoolDiscovery {
    /// Whether to look up mempool transactions with Coinset
    #[cfg_attr(feature = "openapi", schema(example = true))]
    pub mempool_discovery: bool,
}

/// Set target number of peers to maintain
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Peers",
        description = "Set the number of peers Sage tries to stay connected to.",
        response_type = "EmptyResponse"
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
        tag = "Network",
        description = "Set the default network, which wallets without their own network use, and reconnect to it.",
        response_type = "EmptyResponse"
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
        tag = "Network",
        description = "Set the network a wallet uses instead of the default network, or clear it.",
        response_type = "EmptyResponse"
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
        tag = "Network",
        description = "List the networks Sage can connect to."
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
        tag = "Network",
        description = "Get the network the active wallet uses, and whether it's mainnet, testnet11 or another network."
    )
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetNetwork {}

/// Response containing current network information
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "Network"))]
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
        tag = "Network",
        description = "Set whether wallets, by default, sync only the coin changes since they were last synced, instead of all their coins. This applies the next time a wallet is opened.",
        response_type = "EmptyResponse"
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
        tag = "Network",
        description = "Set whether a wallet uses delta sync, instead of the default setting, or clear it.",
        response_type = "EmptyResponse"
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
        description = "Set an address that a wallet's change is sent to, instead of one of its own derived addresses, or clear it. For a BLS wallet, it's also returned as the receive address. The address isn't validated first, and an invalid one stops the wallet from opening until it's cleared.",
        response_type = "EmptyResponse"
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

/// Set how long every wallet's transactions can be confirmed for after they're created
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Transactions",
        description = "Set how many seconds after it's created a transaction can still be confirmed, which must be at least 300 (5 minutes), or 0 for no limit. Offers have their own expiration, and aren't affected.",
        response_type = "EmptyResponse"
    )
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SetTransactionExpiration {
    /// Seconds a transaction can be confirmed for, at least 300, or 0 for no limit
    #[cfg_attr(feature = "openapi", schema(example = 3600))]
    pub seconds: u64,
}

/// Get the UI theme the user last selected
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Themes",
        description = "Get the name of the UI theme that was last selected."
    )
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetSelectedTheme {}

/// Response with the selected UI theme
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "Themes"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetSelectedThemeResponse {
    /// Theme name, if one has been selected
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub theme: Option<String>,
}

/// Remember the UI theme the user selected
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "Themes",
        description = "Set or clear the name of the selected UI theme.",
        response_type = "EmptyResponse"
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SetSelectedTheme {
    /// Theme name (null to clear)
    #[cfg_attr(feature = "openapi", schema(example = "dark"))]
    pub theme: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct EmptyResponse {}

pub type AddPeerResponse = EmptyResponse;
pub type RemovePeerResponse = EmptyResponse;
pub type SetDiscoverPeersResponse = EmptyResponse;
pub type SetMempoolDiscoveryResponse = EmptyResponse;
pub type SetTargetPeersResponse = EmptyResponse;
pub type SetNetworkResponse = EmptyResponse;
pub type SetNetworkOverrideResponse = EmptyResponse;
pub type SetDeltaSyncResponse = EmptyResponse;
pub type SetDeltaSyncOverrideResponse = EmptyResponse;
pub type SetChangeAddressResponse = EmptyResponse;
pub type SetTransactionExpirationResponse = EmptyResponse;
pub type SetSelectedThemeResponse = EmptyResponse;
