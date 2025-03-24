use std::time::Duration;

use itertools::Itertools;
use sage_api::{
    AddPeer, AddPeerResponse, GetNetwork, GetNetworkResponse, GetNetworks, GetNetworksResponse,
    GetPeers, GetPeersResponse, NetworkKind, PeerRecord, RemovePeer, RemovePeerResponse,
    SetDiscoverPeers, SetDiscoverPeersResponse, SetNetwork, SetNetworkResponse, SetTargetPeers,
    SetTargetPeersResponse,
};
use sage_config::{MAINNET, TESTNET11};
use sage_wallet::SyncCommand;

use crate::{Result, Sage};

impl Sage {
    pub async fn get_peers(&self, _req: GetPeers) -> Result<GetPeersResponse> {
        let peer_state = self.peer_state.lock().await;

        Ok(GetPeersResponse {
            peers: peer_state
                .peers_with_heights()
                .into_iter()
                .sorted_by_key(|info| info.0.socket_addr().ip())
                .map(|info| PeerRecord {
                    ip_addr: info.0.socket_addr().ip().to_string(),
                    port: info.0.socket_addr().port(),
                    peak_height: info.1,
                })
                .collect(),
        })
    }

    pub async fn remove_peer(&self, req: RemovePeer) -> Result<RemovePeerResponse> {
        let mut peer_state = self.peer_state.lock().await;

        let ip = req.ip.parse()?;

        if req.ban {
            peer_state.ban(ip, Duration::from_secs(60 * 60), "manually banned");
        } else {
            peer_state.remove_peer(ip);
        }

        Ok(RemovePeerResponse {})
    }

    pub async fn add_peer(&self, req: AddPeer) -> Result<AddPeerResponse> {
        self.command_sender
            .send(SyncCommand::ConnectPeer {
                ip: req.ip.parse()?,
            })
            .await?;

        Ok(AddPeerResponse {})
    }

    pub async fn set_discover_peers(
        &mut self,
        req: SetDiscoverPeers,
    ) -> Result<SetDiscoverPeersResponse> {
        if self.config.network.discover_peers != req.discover_peers {
            self.config.network.discover_peers = req.discover_peers;
            self.save_config()?;
            self.command_sender
                .send(SyncCommand::SetDiscoverPeers(req.discover_peers))
                .await?;
        }

        Ok(SetDiscoverPeersResponse {})
    }

    pub async fn set_target_peers(
        &mut self,
        req: SetTargetPeers,
    ) -> Result<SetTargetPeersResponse> {
        self.config.network.target_peers = req.target_peers;
        self.save_config()?;
        self.command_sender
            .send(SyncCommand::SetTargetPeers(req.target_peers as usize))
            .await?;

        Ok(SetTargetPeersResponse {})
    }

    pub async fn set_network(&mut self, req: SetNetwork) -> Result<SetNetworkResponse> {
        self.config.network.default_network.clone_from(&req.name);

        self.save_config()?;

        let network = self.network();

        self.command_sender
            .send(SyncCommand::SwitchNetwork {
                network_id: network.name.clone(),
                network: chia_wallet_sdk::client::Network {
                    default_port: network.default_port,
                    genesis_challenge: network.genesis_challenge,
                    dns_introducers: network.dns_introducers.clone(),
                },
            })
            .await?;

        self.switch_wallet().await?;
        self.setup_peers().await?;

        Ok(SetNetworkResponse {})
    }

    pub fn get_networks(&mut self, _req: GetNetworks) -> Result<GetNetworksResponse> {
        Ok(self.network_list.clone())
    }

    pub fn get_network(&mut self, _req: GetNetwork) -> Result<GetNetworkResponse> {
        let network = self.network();

        Ok(GetNetworkResponse {
            network: network.clone(),
            kind: if network.genesis_challenge == MAINNET.genesis_challenge {
                NetworkKind::Mainnet
            } else if network.genesis_challenge == TESTNET11.genesis_challenge {
                NetworkKind::Testnet
            } else {
                NetworkKind::Unknown
            },
        })
    }
}
