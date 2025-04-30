use std::time::Duration;

use itertools::Itertools;
use sage_api::{
    AddPeer, AddPeerResponse, GetNetwork, GetNetworkResponse, GetNetworks, GetNetworksResponse,
    GetPeers, GetPeersResponse, NetworkKind, PeerRecord, RemovePeer, RemovePeerResponse,
    SetDiscoverPeers, SetDiscoverPeersResponse, SetNetwork, SetNetworkOverride,
    SetNetworkOverrideResponse, SetNetworkResponse, SetTargetPeers, SetTargetPeersResponse,
};
use sage_config::{MAINNET, TESTNET11};
use sage_wallet::SyncCommand;

use crate::{Error, Result, Sage};

impl Sage {
    pub async fn get_peers(&self, _req: GetPeers) -> Result<GetPeersResponse> {
        let peer_state = self.peer_state.lock().await;

        Ok(GetPeersResponse {
            peers: peer_state
                .peers_with_heights()
                .into_iter()
                .sorted_by_key(|info| info.0.socket_addr().ip())
                .map(|info| {
                    let ip = info.0.socket_addr().ip();
                    PeerRecord {
                        ip_addr: ip.to_string(),
                        port: info.0.socket_addr().port(),
                        peak_height: info.1,
                        user_managed: peer_state.peer(ip).is_some_and(|p| p.user_managed),
                    }
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
                user_managed: true,
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
        self.switch_wallet().await?;
        self.setup_peers().await?;
        Ok(SetNetworkResponse {})
    }

    pub async fn set_network_override(
        &mut self,
        req: SetNetworkOverride,
    ) -> Result<SetNetworkOverrideResponse> {
        let config = self
            .wallet_config
            .wallets
            .iter_mut()
            .find(|w| w.fingerprint == req.fingerprint)
            .ok_or(Error::UnknownFingerprint)?;

        config.network = req.name;

        self.save_config()?;
        self.switch_wallet().await?;
        self.setup_peers().await?;

        Ok(SetNetworkOverrideResponse {})
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
