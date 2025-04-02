use std::{
    collections::HashMap,
    net::IpAddr,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use chia::protocol::Bytes32;
use itertools::Itertools;
use tokio::task::JoinHandle;
use tracing::debug;

use crate::WalletPeer;

#[derive(Debug)]
pub struct PeerInfo {
    pub peer: WalletPeer,
    pub claimed_peak: u32,
    pub header_hash: Bytes32,
    pub receive_message_task: JoinHandle<()>,
    pub user_managed: bool,
}

impl Drop for PeerInfo {
    fn drop(&mut self) {
        self.receive_message_task.abort();
    }
}

#[derive(Debug, Default)]
pub struct PeerState {
    peers: HashMap<IpAddr, PeerInfo>,
    banned_peers: HashMap<IpAddr, u64>,
}

impl PeerState {
    pub fn reset(&mut self) {
        self.peers.clear();
        self.banned_peers.clear();
    }

    pub fn peak(&self) -> Option<(u32, Bytes32)> {
        self.peers
            .values()
            .map(|peer| (peer.claimed_peak, peer.header_hash))
            .sorted_by_key(|(height, _)| -i64::from(*height))
            .next()
    }

    pub fn peak_of(&self, ip: IpAddr) -> Option<(u32, Bytes32)> {
        self.peers
            .get(&ip)
            .map(|peer| (peer.claimed_peak, peer.header_hash))
    }

    pub fn peers(&self) -> Vec<WalletPeer> {
        self.peers.values().map(|info| info.peer.clone()).collect()
    }

    pub fn auto_discovered_peers(&self) -> Vec<WalletPeer> {
        self.peers
            .values()
            .filter(|info| !info.user_managed)
            .map(|info| info.peer.clone())
            .collect()
    }

    pub fn user_managed_peers(&self) -> Vec<WalletPeer> {
        self.peers
            .values()
            .filter(|info| info.user_managed)
            .map(|info| info.peer.clone())
            .collect()
    }

    pub fn peers_with_heights(&self) -> Vec<(WalletPeer, u32)> {
        self.peers
            .values()
            .map(|info| (info.peer.clone(), info.claimed_peak))
            .collect()
    }

    pub fn peer_count(&self) -> usize {
        self.peers.len()
    }

    pub fn is_connected(&self, ip: IpAddr) -> bool {
        self.peers.contains_key(&ip)
    }

    pub fn acquire_peer(&self) -> Option<WalletPeer> {
        self.peers
            .values()
            .max_by_key(|info| info.claimed_peak)
            .map(|info| info.peer.clone())
    }

    pub fn ban(&mut self, ip: IpAddr, duration: Duration, message: &str) {
        debug!("Banning peer {ip} ({duration:?}): {message}");

        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");

        self.banned_peers
            .insert(ip, since_the_epoch.as_secs() + duration.as_secs());

        self.banned_peers
            .retain(|_, ban_until| *ban_until > since_the_epoch.as_secs());

        self.remove_peer(ip);
    }

    pub fn is_banned(&self, ip: IpAddr) -> bool {
        self.banned_peers.get(&ip).is_some_and(|ban_until| {
            let start = SystemTime::now();
            let since_the_epoch = start
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards");

            *ban_until > since_the_epoch.as_secs()
        })
    }

    pub fn update_peak(&mut self, ip: IpAddr, height: u32, header_hash: Bytes32) {
        if let Some(peer) = self.peers.get_mut(&ip) {
            peer.claimed_peak = height;
            peer.header_hash = header_hash;
        }
    }

    pub fn peer(&self, ip: IpAddr) -> Option<&PeerInfo> {
        self.peers.get(&ip)
    }

    pub fn remove_peer(&mut self, ip: IpAddr) {
        self.peers.remove(&ip);
    }

    pub(super) fn add_peer(&mut self, state: PeerInfo) {
        self.peers.insert(state.peer.socket_addr().ip(), state);
    }

    pub fn banned_peers(&mut self) -> &HashMap<IpAddr, u64> {
        self.banned_peers.retain(|_, ban_until| {
            let start = SystemTime::now();
            let since_the_epoch = start
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards");

            *ban_until > since_the_epoch.as_secs()
        });
        &self.banned_peers
    }
}
