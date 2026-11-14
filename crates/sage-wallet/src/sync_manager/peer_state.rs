use std::{
    cmp::Reverse,
    collections::HashMap,
    net::IpAddr,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use chia_wallet_sdk::prelude::*;
use itertools::Itertools;
use tokio::{task::JoinHandle, time::Instant};
use tracing::debug;

use crate::WalletPeer;

/// How many blocks behind the highest peak a peer that sends mempool updates can be, and still
/// be preferred for syncing the wallet.
const MAX_SYNC_PEER_LAG: u32 = 3;

#[derive(Debug)]
pub struct PeerInfo {
    pub peer: WalletPeer,
    pub claimed_peak: u32,
    pub header_hash: Bytes32,
    /// When the peer last sent a new peak, or connected if it hasn't sent one since.
    pub last_peak_at: Instant,
    pub receive_message_task: JoinHandle<()>,
    pub user_managed: bool,
    /// Whether the peer announces the wallet's mempool transactions (CHIP-26). Nodes older than
    /// that don't, so the wallet would only see its transactions once they're confirmed.
    pub sends_mempool_updates: bool,
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
        self.peers_by_peak().next().cloned()
    }

    /// The peer to subscribe the wallet with. Only that peer announces the wallet's mempool
    /// transactions, so one that sends mempool updates is preferred, as long as it's within
    /// a few blocks of the highest peak.
    pub fn acquire_sync_peer(&self) -> Option<WalletPeer> {
        let highest_peak = self.peers.values().map(|info| info.claimed_peak).max()?;

        self.peers
            .values()
            .max_by_key(|info| {
                let close_to_peak =
                    info.claimed_peak.saturating_add(MAX_SYNC_PEER_LAG) >= highest_peak;
                (
                    info.sends_mempool_updates && close_to_peak,
                    info.claimed_peak,
                )
            })
            .map(|info| info.peer.clone())
    }

    pub(super) fn set_sends_mempool_updates(&mut self, ip: IpAddr) {
        if let Some(info) = self.peers.get_mut(&ip) {
            info.sends_mempool_updates = true;
        }
    }

    /// Every peer, the one claiming the highest peak first.
    pub fn peers_by_peak(&self) -> impl Iterator<Item = &WalletPeer> {
        self.peers
            .values()
            .sorted_by_key(|info| Reverse(info.claimed_peak))
            .map(|info| &info.peer)
    }

    /// Auto-discovered peers that haven't sent a new peak within `max_silence`. A connection can
    /// die without closing, which leaves a peer connected that will never answer. User-managed
    /// peers are left alone, since the user chose them.
    pub fn silent_peers(&self, now: Instant, max_silence: Duration) -> Vec<IpAddr> {
        self.peers
            .iter()
            .filter(|(_, info)| {
                !info.user_managed
                    && now.saturating_duration_since(info.last_peak_at) >= max_silence
            })
            .map(|(&ip, _)| ip)
            .collect()
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
            peer.last_peak_at = Instant::now();
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

    /// Adds a peer under an IP other than its connection's, since every local test peer
    /// connects on 127.0.0.1.
    #[cfg(test)]
    pub(crate) fn add_peer_at(&mut self, ip: IpAddr, peer: WalletPeer) {
        self.peers.insert(
            ip,
            PeerInfo {
                peer,
                claimed_peak: 0,
                header_hash: Bytes32::default(),
                last_peak_at: Instant::now(),
                receive_message_task: tokio::spawn(async {}),
                user_managed: false,
                sends_mempool_updates: false,
            },
        );
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

#[cfg(test)]
mod tests {
    use tokio::time::sleep;

    use crate::unresponsive_peer;

    use super::*;

    const MAX_SILENCE: Duration = Duration::from_mins(5);

    /// Adds a peer under the IP 10.0.0.`last_octet`. Its connection is on 127.0.0.1 regardless,
    /// since macOS only routes that loopback address.
    async fn add_peer(
        state: &mut PeerState,
        last_octet: u8,
        claimed_peak: u32,
        last_peak_at: Instant,
        user_managed: bool,
    ) -> anyhow::Result<IpAddr> {
        let ip = IpAddr::from([10, 0, 0, last_octet]);
        state.peers.insert(
            ip,
            PeerInfo {
                peer: unresponsive_peer().await?,
                claimed_peak,
                header_hash: Bytes32::default(),
                last_peak_at,
                receive_message_task: tokio::spawn(async {}),
                user_managed,
                sends_mempool_updates: false,
            },
        );
        Ok(ip)
    }

    #[tokio::test]
    async fn peers_by_peak_orders_highest_peak_first() -> anyhow::Result<()> {
        let now = Instant::now();
        let mut state = PeerState::default();
        let lowest = add_peer(&mut state, 2, 10, now, false).await?;
        let highest = add_peer(&mut state, 3, 30, now, false).await?;
        let middle = add_peer(&mut state, 4, 20, now, false).await?;

        let addr = |ip| state.peer(ip).map(|info| info.peer.socket_addr());

        let addrs = state
            .peers_by_peak()
            .map(|peer| Some(peer.socket_addr()))
            .collect_vec();

        assert_eq!(addrs, [addr(highest), addr(middle), addr(lowest)]);

        Ok(())
    }

    #[tokio::test]
    async fn sync_peer_prefers_peers_that_send_mempool_updates() -> anyhow::Result<()> {
        let now = Instant::now();
        let mut state = PeerState::default();
        let addr = |state: &PeerState, ip| state.peer(ip).map(|info| info.peer.socket_addr());
        let sync_addr = |state: &PeerState| state.acquire_sync_peer().map(|p| p.socket_addr());

        // Without any that send mempool updates, the highest peak wins.
        let old = add_peer(&mut state, 2, 30, now, false).await?;
        let older = add_peer(&mut state, 3, 25, now, false).await?;
        assert_eq!(sync_addr(&state), addr(&state, old));

        // One that's too far behind isn't preferred.
        state.set_sends_mempool_updates(older);
        assert_eq!(sync_addr(&state), addr(&state, old));

        // One a few blocks behind is.
        let new = add_peer(&mut state, 4, 30 - MAX_SYNC_PEER_LAG, now, false).await?;
        state.set_sends_mempool_updates(new);
        assert_eq!(sync_addr(&state), addr(&state, new));

        // Among those, the highest peak wins.
        let newer = add_peer(&mut state, 5, 29, now, false).await?;
        state.set_sends_mempool_updates(newer);
        assert_eq!(sync_addr(&state), addr(&state, newer));

        Ok(())
    }

    // These tests check silence at a later time rather than backdating peaks, since subtracting
    // from an `Instant` panics when the machine booted recently.

    #[tokio::test]
    async fn peer_that_stops_sending_peaks_is_silent() -> anyhow::Result<()> {
        let start = Instant::now();
        let mut state = PeerState::default();
        let silent = add_peer(&mut state, 2, 100, start, false).await?;
        add_peer(&mut state, 3, 100, start + MAX_SILENCE / 2, false).await?;

        assert_eq!(
            state.silent_peers(start + MAX_SILENCE, MAX_SILENCE),
            [silent]
        );

        Ok(())
    }

    #[tokio::test]
    async fn user_managed_peers_are_never_silent() -> anyhow::Result<()> {
        let start = Instant::now();
        let mut state = PeerState::default();
        add_peer(&mut state, 2, 100, start, true).await?;

        assert!(
            state
                .silent_peers(start + MAX_SILENCE, MAX_SILENCE)
                .is_empty()
        );

        Ok(())
    }

    #[tokio::test]
    async fn new_peak_refreshes_a_peer() -> anyhow::Result<()> {
        let start = Instant::now();
        let mut state = PeerState::default();
        let ip = add_peer(&mut state, 2, 100, start, false).await?;
        assert_eq!(state.silent_peers(start + MAX_SILENCE, MAX_SILENCE), [ip]);

        sleep(Duration::from_millis(10)).await;
        state.update_peak(ip, 101, Bytes32::default());

        assert!(
            state
                .silent_peers(start + MAX_SILENCE, MAX_SILENCE)
                .is_empty()
        );

        Ok(())
    }
}
