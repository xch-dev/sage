use std::{
    collections::{HashMap, HashSet},
    net::IpAddr,
};

use chia::protocol::Bytes32;
use chia_wallet_sdk::Peer;
use itertools::Itertools;
use tokio::task::JoinHandle;

#[derive(Debug)]
pub struct PeerInfo {
    pub peer: Peer,
    pub claimed_peak: u32,
    pub header_hash: Bytes32,
    pub receive_message_task: JoinHandle<()>,
}

impl Drop for PeerInfo {
    fn drop(&mut self) {
        self.receive_message_task.abort();
    }
}

#[derive(Debug, Default)]
pub struct PeerState {
    peers: HashMap<IpAddr, PeerInfo>,
    banned_peers: HashSet<IpAddr>,
    trusted_peers: HashSet<IpAddr>,
}

impl PeerState {
    pub fn reset(&mut self) {
        self.peers.clear();
        self.banned_peers.clear();
        self.trusted_peers.clear();
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

    pub fn peers(&self) -> impl Iterator<Item = &PeerInfo> {
        self.peers.values()
    }

    pub fn peer_count(&self) -> usize {
        self.peers.len()
    }

    pub fn is_connected(&self, ip: IpAddr) -> bool {
        self.peers.contains_key(&ip)
    }

    pub fn acquire_peer(&self) -> Option<Peer> {
        self.peers
            .values()
            .max_by_key(|peer| peer.claimed_peak)
            .map(|peer| peer.peer.clone())
    }

    pub fn ban(&mut self, ip: IpAddr) {
        if self.trusted_peers.contains(&ip) {
            return;
        }
        self.banned_peers.insert(ip);
        self.remove_peer(ip);
    }

    pub fn is_banned(&self, ip: IpAddr) -> bool {
        self.banned_peers.contains(&ip)
    }

    pub fn trust(&mut self, ip: IpAddr) {
        self.trusted_peers.insert(ip);
        self.banned_peers.remove(&ip);
    }

    pub fn update_peak(&mut self, ip: IpAddr, height: u32, header_hash: Bytes32) {
        if let Some(peer) = self.peers.get_mut(&ip) {
            peer.claimed_peak = height;
            peer.header_hash = header_hash;
        }
    }

    pub fn remove_peer(&mut self, ip: IpAddr) {
        self.peers.remove(&ip);
    }

    pub(super) fn add_peer(&mut self, state: PeerInfo) {
        self.peers.insert(state.peer.socket_addr().ip(), state);
    }
}
