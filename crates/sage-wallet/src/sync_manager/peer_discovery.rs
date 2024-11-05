use std::{
    cmp::Reverse,
    net::{IpAddr, SocketAddr},
    str::FromStr,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use chia::{
    protocol::{Message, NewPeakWallet, ProtocolMessageTypes},
    traits::Streamable,
};
use chia_wallet_sdk::{connect_peer, Peer};
use futures_lite::StreamExt;
use futures_util::stream::FuturesUnordered;
use itertools::Itertools;
use tokio::{sync::mpsc, time::timeout};
use tracing::{debug, info, warn};

use crate::SyncCommand;

use super::{PeerInfo, SyncManager};

impl SyncManager {
    pub(super) async fn clear_subscriptions(&self) {
        let mut futures = FuturesUnordered::new();

        for peer in self
            .state
            .lock()
            .await
            .peers()
            .map(|info| info.peer.clone())
            .collect_vec()
        {
            let ip = peer.socket_addr().ip();

            let puzzle_peer = peer.clone();
            let duration = self.options.remove_subscription_timeout;

            futures.push(async move {
                match timeout(duration, puzzle_peer.remove_puzzle_subscriptions(None)).await {
                    Ok(Ok(..)) => {}
                    Ok(Err(error)) => {
                        debug!("Failed to clear puzzle subscriptions from {ip}: {error}");
                    }
                    Err(_timeout) => {
                        debug!("Timeout clearing puzzle subscriptions from {ip}");
                    }
                }

                match timeout(duration, peer.remove_coin_subscriptions(None)).await {
                    Ok(Ok(..)) => {}
                    Ok(Err(error)) => {
                        debug!("Failed to clear coin subscriptions from {ip}: {error}");
                    }
                    Err(_timeout) => {
                        debug!("Timeout clearing coin subscriptions from {ip}");
                    }
                }
            });
        }

        while let Some(()) = futures.next().await {}
    }

    pub(super) async fn dns_discovery(&mut self) {
        let addrs = self
            .network
            .lookup_all(self.options.dns_timeout, self.options.dns_batch_size)
            .await;

        for addrs in addrs.chunks(self.options.connection_batch_size) {
            if self.connect_batch(addrs).await {
                break;
            }
        }
    }

    pub(super) async fn peer_discovery(&mut self) -> bool {
        let peers: Vec<Peer> = self
            .state
            .lock()
            .await
            .peers()
            .map(|info| info.peer.clone())
            .collect();

        if peers.is_empty() {
            warn!("No existing peers to request new peers from");
            return false;
        }

        let mut futures = FuturesUnordered::new();

        for peer in peers {
            let ip = peer.socket_addr().ip();
            let duration = self.options.request_peers_timeout;
            futures.push(async move {
                let result = timeout(duration, peer.request_peers()).await;
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
                Ok(Ok(mut response)) => {
                    response.peer_list.retain(|item| {
                        item.timestamp >= timestamp - self.options.max_peer_age_seconds
                    });

                    if !response.peer_list.is_empty() {
                        info!(
                            "Received {} recent peers from {}",
                            response.peer_list.len(),
                            ip
                        );
                    }

                    response
                        .peer_list
                        .sort_by_key(|item| Reverse(item.timestamp));

                    let mut addrs = Vec::new();

                    for item in response.peer_list {
                        let Some(new_ip) = IpAddr::from_str(&item.host).ok() else {
                            debug!("Invalid IP address in peer list");
                            self.state.lock().await.ban(
                                ip,
                                Duration::from_secs(300),
                                "invalid ip in peer list",
                            );
                            break;
                        };
                        addrs.push(SocketAddr::new(new_ip, self.network.default_port));
                    }

                    for addrs in addrs.chunks(self.options.connection_batch_size) {
                        if self.connect_batch(addrs).await {
                            return true;
                        }
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

    pub(super) async fn connect_batch(&mut self, addrs: &[SocketAddr]) -> bool {
        let mut futures = FuturesUnordered::new();

        for &socket_addr in addrs {
            let state = self.state.lock().await;
            if state.is_connected(socket_addr.ip()) || state.is_banned(socket_addr.ip()) {
                continue;
            }
            drop(state);

            let network_id = self.network_id.clone();
            let connector = self.connector.clone();
            let duration = self.options.connection_timeout;

            futures.push(async move {
                let result =
                    timeout(duration, connect_peer(network_id, connector, socket_addr)).await;
                (socket_addr, result)
            });
        }

        while let Some((socket_addr, result)) = futures.next().await {
            match result {
                Ok(Ok((peer, receiver))) => {
                    if self.try_add_peer(peer, receiver).await {
                        if self.check_peer_count().await {
                            return true;
                        }
                    } else {
                        self.state.lock().await.ban(
                            socket_addr.ip(),
                            Duration::from_secs(60 * 10),
                            "could not add peer",
                        );
                    }
                }
                Ok(Err(error)) => {
                    debug!("Failed to connect to peer {socket_addr}: {error}");
                    self.state.lock().await.ban(
                        socket_addr.ip(),
                        Duration::from_secs(60 * 10),
                        "failed to connect",
                    );
                }
                Err(_timeout) => {
                    debug!("Connection to peer {socket_addr} timed out");
                    self.state.lock().await.ban(
                        socket_addr.ip(),
                        Duration::from_secs(60 * 10),
                        "connection timed out",
                    );
                }
            }
        }

        self.check_peer_count().await
    }

    async fn check_peer_count(&mut self) -> bool {
        self.state.lock().await.peer_count() >= self.options.target_peers
    }

    async fn try_add_peer(&mut self, peer: Peer, mut receiver: mpsc::Receiver<Message>) -> bool {
        let Ok(Some(message)) = timeout(self.options.initial_peak_timeout, receiver.recv()).await
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

        self.state.lock().await.add_peer(PeerInfo {
            peer,
            claimed_peak: message.height,
            header_hash: message.header_hash,
            receive_message_task: tokio::spawn(async move {
                while let Some(message) = receiver.recv().await {
                    debug!("Received message from peer {}: {:?}", ip, message.msg_type);

                    if sender
                        .send(SyncCommand::HandleMessage { ip, message })
                        .await
                        .is_err()
                    {
                        return;
                    }
                }

                sender.send(SyncCommand::ConnectionClosed(ip)).await.ok();
            }),
        });

        true
    }
}
