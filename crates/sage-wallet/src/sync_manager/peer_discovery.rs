use std::{
    cmp::Reverse,
    net::{IpAddr, SocketAddr},
    str::FromStr,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use chia::{
    protocol::{
        ChiaProtocolMessage, Handshake, Message, NewPeakWallet, NodeType, ProtocolMessageTypes,
        TimestampedPeerInfo,
    },
    traits::Streamable,
};
use chia_streamable_macro::Streamable;
use chia_wallet_sdk::client::{connect_peer, ClientError, Peer, PeerOptions};
use futures_lite::StreamExt;
use futures_util::stream::FuturesUnordered;
use rand::Rng;
use tokio::{sync::mpsc, time::timeout};
use tracing::{debug, info, warn};

use crate::{SyncCommand, WalletError, WalletPeer};

use super::{dns::lookup_all, PeerInfo, SyncManager};

#[derive(Streamable)]
struct RequestPeersIntroducer {}

impl ChiaProtocolMessage for RequestPeersIntroducer {
    fn msg_type() -> ProtocolMessageTypes {
        ProtocolMessageTypes::RequestPeersIntroducer
    }
}

#[derive(Streamable)]
struct RespondPeersIntroducer {
    peer_list: Vec<TimestampedPeerInfo>,
}

impl ChiaProtocolMessage for RespondPeersIntroducer {
    fn msg_type() -> ProtocolMessageTypes {
        ProtocolMessageTypes::RespondPeersIntroducer
    }
}

impl SyncManager {
    pub(super) async fn clear_subscriptions(&self) {
        let mut futures = FuturesUnordered::new();

        for peer in self.state.lock().await.peers() {
            let ip = peer.socket_addr().ip();

            let puzzle_peer = peer.clone();
            let duration = self.options.timeouts.remove_subscription;

            futures.push(async move {
                match timeout(duration, puzzle_peer.unsubscribe()).await {
                    Ok(Ok(..)) => {}
                    Ok(Err(error)) => {
                        debug!("Failed to clear subscriptions from {ip}: {error}");
                    }
                    Err(_timeout) => {
                        debug!("Timeout clearing subscriptions from {ip}");
                    }
                }
            });
        }

        while let Some(()) = futures.next().await {}
    }

    pub(super) async fn dns_discovery(&mut self) -> bool {
        let addrs = lookup_all(
            &self.network.dns_introducers(),
            self.network.default_port,
            self.options.timeouts.dns,
            self.options.dns_batch_size,
        )
        .await;

        if addrs.is_empty() {
            return false;
        }

        for addrs in addrs.chunks(self.options.connection_batch_size) {
            if self.connect_batch(addrs, false, false).await {
                break;
            }
        }

        true
    }

    pub(super) async fn introducer_discovery(&mut self) {
        info!(
            "Looking up non-DNS introducers {}",
            self.network.peer_introducers().join(", ")
        );

        let mut futures = FuturesUnordered::new();

        for host in self.network.peer_introducers() {
            let introducer_timeout = self.options.timeouts.introducer;
            let port = self.network.default_port;
            let connector = self.connector.clone();
            let network_id = self.network.network_id();

            futures.push(async move {
                let host_clone = host.clone();

                let result = timeout(introducer_timeout, async move {
                    let (peer, mut receiver) = Peer::connect_full_uri(
                        &format!("wss://{host_clone}:{port}/ws"),
                        connector,
                        PeerOptions::default(),
                    )
                    .await?;

                    peer.send(Handshake {
                        network_id: network_id.clone(),
                        protocol_version: "0.0.37".to_string(),
                        software_version: "0.0.0".to_string(),
                        server_port: 0,
                        node_type: NodeType::Wallet,
                        capabilities: vec![
                            (1, "1".to_string()),
                            (2, "1".to_string()),
                            (3, "1".to_string()),
                        ],
                    })
                    .await?;

                    let Some(message) = receiver.recv().await else {
                        return Err(ClientError::MissingHandshake)?;
                    };

                    if message.msg_type != ProtocolMessageTypes::Handshake {
                        return Err(ClientError::InvalidResponse(
                            vec![ProtocolMessageTypes::Handshake],
                            message.msg_type,
                        ))?;
                    }

                    let handshake =
                        Handshake::from_bytes(&message.data).map_err(ClientError::from)?;

                    if handshake.node_type != NodeType::Introducer {
                        return Err(ClientError::WrongNodeType(
                            NodeType::Introducer,
                            handshake.node_type,
                        ))?;
                    }

                    if handshake.network_id != network_id {
                        return Err(ClientError::WrongNetwork(
                            network_id.to_string(),
                            handshake.network_id,
                        ))?;
                    }

                    let peer_list = peer
                        .request_infallible::<RespondPeersIntroducer, _>(RequestPeersIntroducer {})
                        .await?
                        .peer_list;

                    Result::<_, WalletError>::Ok((peer.socket_addr().ip(), peer_list))
                })
                .await;

                (host, result)
            });
        }

        while let Some((host, result)) = futures.next().await {
            match result {
                Ok(Ok((ip, peer_list))) => {
                    self.handle_peer_list(None, peer_list, ip, false).await;
                }
                Ok(Err(error)) => {
                    debug!("Failed to request peers from {host}: {error}");
                }
                Err(_timeout) => {
                    debug!("Timeout requesting peers from {host}");
                }
            }
        }
    }

    pub(super) async fn peer_discovery(&mut self) -> bool {
        let peers = self.state.lock().await.peers();

        if peers.is_empty() {
            warn!("No existing peers to request new peers from");
            return false;
        }

        let mut futures = FuturesUnordered::new();

        for peer in peers {
            let ip = peer.socket_addr().ip();
            let duration = self.options.timeouts.request_peers;
            futures.push(async move {
                let result = timeout(duration, peer.request_peers())
                    .await
                    .map(|result| result.map(|result| result.peer_list));
                (ip, result)
            });
        }

        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        let timestamp = since_the_epoch.as_secs();

        while let Some((ip, result)) = futures.next().await {
            match result {
                Ok(Ok(peer_list)) => {
                    if self
                        .handle_peer_list(Some(timestamp), peer_list, ip, true)
                        .await
                    {
                        return true;
                    }
                }
                Ok(Err(error)) => {
                    debug!("Failed to request peers from {}: {}", ip, error);
                    self.state.lock().await.ban(
                        ip,
                        Duration::from_secs(300),
                        "failed to request peers",
                    );
                }
                Err(_timeout) => {}
            }
        }

        false
    }

    async fn handle_peer_list(
        &mut self,
        timestamp: Option<u64>,
        mut peer_list: Vec<TimestampedPeerInfo>,
        ip: IpAddr,
        ban: bool,
    ) -> bool {
        if let Some(timestamp) = timestamp {
            peer_list
                .retain(|item| item.timestamp >= timestamp - self.options.max_peer_age_seconds);
        }

        if !peer_list.is_empty() {
            info!("Received {} recent peers from {}", peer_list.len(), ip);
        }

        if timestamp.is_some() {
            peer_list.sort_by_key(|item| Reverse(item.timestamp));
        }

        let mut addrs = Vec::new();

        for item in peer_list {
            let Some(new_ip) = IpAddr::from_str(&item.host).ok() else {
                debug!("Invalid IP address in peer list");

                if ban {
                    self.state.lock().await.ban(
                        ip,
                        Duration::from_secs(300),
                        "invalid ip in peer list",
                    );
                }

                break;
            };
            addrs.push(SocketAddr::new(new_ip, self.network.default_port));
        }

        for addrs in addrs.chunks(self.options.connection_batch_size) {
            if self.connect_batch(addrs, false, false).await {
                return true;
            }
        }

        false
    }

    pub(super) async fn connect_batch(
        &mut self,
        addrs: &[SocketAddr],
        force: bool,
        user_managed: bool,
    ) -> bool {
        let mut futures = FuturesUnordered::new();

        for &socket_addr in addrs {
            let state = self.state.lock().await;
            if state.is_connected(socket_addr.ip()) || state.is_banned(socket_addr.ip()) {
                continue;
            }
            drop(state);

            let network_id = self.network.network_id();
            let connector = self.connector.clone();
            let duration = self.options.timeouts.connection;

            futures.push(async move {
                let result = timeout(
                    duration,
                    connect_peer(network_id, connector, socket_addr, PeerOptions::default()),
                )
                .await;
                (socket_addr, result)
            });
        }

        while let Some((socket_addr, result)) = futures.next().await {
            match result {
                Ok(Ok((peer, receiver))) => {
                    if self.try_add_peer(peer, receiver, force, user_managed).await {
                        if self.check_peer_count().await {
                            return true;
                        }
                    } else if !force {
                        self.state.lock().await.ban(
                            socket_addr.ip(),
                            Duration::from_secs(60 * 10),
                            "could not add peer",
                        );
                    }
                }
                Ok(Err(error)) => {
                    debug!("Failed to connect to peer {socket_addr}: {error}");
                    if !force {
                        self.state.lock().await.ban(
                            socket_addr.ip(),
                            Duration::from_secs(60 * 10),
                            "failed to connect",
                        );
                    }
                }
                Err(_timeout) => {
                    debug!("Connection to peer {socket_addr} timed out");
                    if !force {
                        self.state.lock().await.ban(
                            socket_addr.ip(),
                            Duration::from_secs(60 * 10),
                            "connection timed out",
                        );
                    }
                }
            }
        }

        self.check_peer_count().await
    }

    async fn check_peer_count(&mut self) -> bool {
        self.state.lock().await.peer_count() >= self.options.target_peers
    }

    pub(crate) async fn try_add_peer(
        &mut self,
        peer: Peer,
        mut receiver: mpsc::Receiver<Message>,
        force: bool,
        user_managed: bool,
    ) -> bool {
        let Ok(Some(message)) = timeout(self.options.timeouts.initial_peak, receiver.recv()).await
        else {
            debug!(
                "Timeout receiving NewPeakWallet message from peer {}",
                peer.socket_addr()
            );
            return false;
        };

        if message.msg_type != ProtocolMessageTypes::NewPeakWallet {
            debug!(
                "Received unexpected message from peer {}",
                peer.socket_addr()
            );
            return false;
        }

        let Ok(message) = NewPeakWallet::from_bytes(&message.data) else {
            debug!(
                "Invalid NewPeakWallet message from peer {}",
                peer.socket_addr()
            );
            return false;
        };

        let ip = peer.socket_addr().ip();
        let sender = self.command_sender.clone();

        let mut state = self.state.lock().await;

        if !force && state.peer_count() >= self.options.target_peers {
            debug!(
                "Peer {} is trying to connect when we have enough peers",
                peer.socket_addr()
            );
            return false;
        } else if force && state.peer_count() >= self.options.target_peers {
            let mut peers = state.peers_with_heights();
            let mut rng = rand::thread_rng();

            // Sort so user managed are deprioritized, then by height, then randomly
            peers.sort_by_key(|(peer, height)| {
                let peer_info = state.peer(peer.socket_addr().ip()).expect("peer not found");
                (peer_info.user_managed, *height, rng.gen_range(0..100))
            });

            let count = state.peer_count() - self.options.target_peers + 1;

            debug!("Removing {} peers to make room for new peer", count);

            for peer in peers.iter().take(count) {
                state.remove_peer(peer.0.socket_addr().ip());
            }
        }

        for (existing_peer, height) in state.peers_with_heights() {
            if message.height < height.saturating_sub(3) {
                debug!(
                    "Peer {} is behind by more than 3 blocks, disconnecting",
                    peer.socket_addr()
                );
                return false;
            } else if message.height > height.saturating_add(3) {
                state.ban(
                    existing_peer.socket_addr().ip(),
                    Duration::from_secs(900),
                    "peer is behind",
                );
            }
        }

        state.add_peer(PeerInfo {
            peer: WalletPeer::new(peer),
            claimed_peak: message.height,
            header_hash: message.header_hash,
            user_managed,
            receive_message_task: tokio::spawn(async move {
                while let Some(message) = receiver.recv().await {
                    debug!("Received message from peer {}: {:?}", ip, message.msg_type);

                    if sender
                        .send(SyncCommand::HandleMessage { ip, message })
                        .await
                        .is_err()
                    {
                        break;
                    }
                }

                sender.send(SyncCommand::ConnectionClosed(ip)).await.ok();
            }),
        });

        true
    }
}
