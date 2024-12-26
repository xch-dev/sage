use std::time::Duration;

use itertools::Itertools;
use sage_api::{
    AddPeer, AddPeerResponse, GetNetworks, GetNetworksResponse, GetPeers, GetPeersResponse,
    PeerRecord, RemovePeer, RemovePeerResponse, SetDerivationBatchSize,
    SetDerivationBatchSizeResponse, SetDeriveAutomatically, SetDeriveAutomaticallyResponse,
    SetDiscoverPeers, SetDiscoverPeersResponse, SetNetworkId, SetNetworkIdResponse, SetTargetPeers,
    SetTargetPeersResponse,
};
use sage_wallet::SyncCommand;

use crate::{parse_genesis_challenge, Result, Sage};

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
                    trusted: false,
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
                trusted: req.trusted,
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

    pub async fn set_network_id(&mut self, req: SetNetworkId) -> Result<SetNetworkIdResponse> {
        self.config.network.network_id.clone_from(&req.network_id);
        self.save_config()?;

        let network = self.network();

        self.command_sender
            .send(SyncCommand::SwitchNetwork {
                network_id: req.network_id,
                network: chia_wallet_sdk::Network {
                    default_port: network.default_port,
                    genesis_challenge: parse_genesis_challenge(network.genesis_challenge.clone())?,
                    dns_introducers: network.dns_introducers.clone(),
                },
            })
            .await?;

        self.switch_wallet().await?;
        self.setup_peers().await?;

        Ok(SetNetworkIdResponse {})
    }

    pub fn set_derive_automatically(
        &mut self,
        req: SetDeriveAutomatically,
    ) -> Result<SetDeriveAutomaticallyResponse> {
        let config = self.try_wallet_config_mut(req.fingerprint);

        if config.derive_automatically != req.derive_automatically {
            config.derive_automatically = req.derive_automatically;
            self.save_config()?;
        }

        Ok(SetDeriveAutomaticallyResponse {})
    }

    pub fn set_derivation_batch_size(
        &mut self,
        req: SetDerivationBatchSize,
    ) -> Result<SetDerivationBatchSizeResponse> {
        let config = self.try_wallet_config_mut(req.fingerprint);
        config.derivation_batch_size = req.derivation_batch_size;
        self.save_config()?;

        // TODO: Update sync manager

        Ok(SetDerivationBatchSizeResponse {})
    }

    pub fn get_networks(&mut self, _req: GetNetworks) -> Result<GetNetworksResponse> {
        Ok(GetNetworksResponse {
            networks: self.networks.clone(),
        })
    }
}
