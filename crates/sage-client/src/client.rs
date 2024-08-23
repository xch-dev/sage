use std::{
    collections::{HashMap, HashSet},
    net::{IpAddr, SocketAddr},
    ops::Deref,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use chia::protocol::Message;
use native_tls::TlsConnector;
use tokio::sync::{mpsc, Mutex};

use crate::{connect_peer, ClientError, Network, Peer};

#[derive(Debug, Clone)]
pub struct Client {
    network_id: String,
    network: Network,
    tls_connector: TlsConnector,
    state: Arc<Mutex<ClientState>>,
}

impl Deref for Client {
    type Target = Mutex<ClientState>;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

#[derive(Debug, Default, Clone)]
pub struct ClientState {
    peers: HashMap<IpAddr, Peer>,
    banned_peers: HashMap<IpAddr, u64>,
    trusted_peers: HashSet<IpAddr>,
}

impl Client {
    pub fn new(network_id: String, network: Network, tls_connector: TlsConnector) -> Self {
        Self {
            network_id,
            network,
            tls_connector,
            state: Arc::new(Mutex::new(ClientState::default())),
        }
    }

    pub fn network(&self) -> &Network {
        &self.network
    }

    pub async fn connect(
        &self,
        socket_addr: SocketAddr,
    ) -> Result<mpsc::Receiver<Message>, ClientError> {
        let (peer, receiver) = connect_peer(
            self.network_id.clone(),
            self.tls_connector.clone(),
            socket_addr,
        )
        .await?;

        let mut state = self.state.lock().await;
        let ip_addr = peer.socket_addr().ip();

        if state.is_banned(&ip_addr) {
            return Err(ClientError::BannedPeer);
        }

        state.peers.insert(peer.socket_addr().ip(), peer);

        Ok(receiver)
    }
}

impl ClientState {
    pub fn peers(&self) -> impl Iterator<Item = &Peer> {
        self.peers.values()
    }

    pub fn disconnect(&mut self, ip_addr: &IpAddr) -> bool {
        self.peers.remove(ip_addr).is_some()
    }

    pub fn is_banned(&self, ip_addr: &IpAddr) -> bool {
        self.banned_peers.contains_key(ip_addr)
    }

    pub fn is_trusted(&self, ip_addr: &IpAddr) -> bool {
        self.trusted_peers.contains(ip_addr)
    }

    pub fn ban(&mut self, ip_addr: IpAddr) -> bool {
        if self.is_trusted(&ip_addr) {
            return false;
        }

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();

        self.disconnect(&ip_addr);
        self.banned_peers.insert(ip_addr, timestamp).is_none()
    }

    pub fn unban(&mut self, ip_addr: IpAddr) -> bool {
        self.banned_peers.remove(&ip_addr).is_some()
    }

    pub fn trust(&mut self, ip_addr: IpAddr) -> bool {
        let result = self.trusted_peers.insert(ip_addr);
        self.banned_peers.remove(&ip_addr);
        result
    }

    pub fn untrust(&mut self, ip_addr: IpAddr) -> bool {
        self.trusted_peers.remove(&ip_addr)
    }
}
