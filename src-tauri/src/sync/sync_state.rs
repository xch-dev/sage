use std::{
    collections::{HashMap, HashSet},
    net::IpAddr,
};

use chia::protocol::Bytes32;
use chia_wallet_sdk::Peer;

use super::peer_state::PeerState;

#[derive(Default)]
pub struct SyncState {
    peers: HashMap<IpAddr, PeerState>,
    banned_peers: HashSet<IpAddr>,
    trusted_peers: HashSet<IpAddr>,
}

impl SyncState {
    pub fn new(banned_peers: HashSet<IpAddr>, trusted_peers: HashSet<IpAddr>) -> Self {
        Self {
            peers: HashMap::new(),
            banned_peers,
            trusted_peers,
        }
    }

    pub fn peak_height(&self) -> u32 {
        self.peers
            .values()
            .map(|peer| peer.claimed_peak)
            .max()
            .unwrap_or(0)
    }

    pub fn peers(&self) -> impl Iterator<Item = &Peer> {
        self.peers.values().map(|peer| &peer.peer)
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

    pub fn reset_peers(&mut self) {
        self.peers.drain().for_each(|(_ip, peer)| {
            peer.task.abort();
        });
    }

    pub fn update_peak(&mut self, ip: IpAddr, height: u32, header_hash: Bytes32) {
        if let Some(peer) = self.peers.get_mut(&ip) {
            peer.claimed_peak = height;
            peer.header_hash = header_hash;
        }
    }

    pub fn remove_peer(&mut self, ip: IpAddr) {
        if let Some(peer) = self.peers.remove(&ip) {
            peer.task.abort();
        }
    }

    pub(super) fn add_peer(&mut self, state: PeerState) {
        if let Some(old) = self.peers.insert(state.peer.socket_addr().ip(), state) {
            old.task.abort();
        }
    }
}
