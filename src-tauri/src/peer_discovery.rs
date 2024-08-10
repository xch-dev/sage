use std::{collections::HashMap, net::SocketAddr, sync::Arc, time::Duration};

use chia::protocol::Message;
use futures_util::{stream::FuturesUnordered, StreamExt};
use native_tls::TlsConnector;
use rand::{seq::SliceRandom, thread_rng};
use sage_client::{connect_peer, Network, Peer};
use tokio::{
    net::lookup_host,
    sync::{mpsc, Mutex},
    time::{sleep, timeout},
};
use tracing::{debug, info, instrument, warn};

use crate::config::NetworkConfig;

#[instrument(skip(tls_connector, peers))]
pub async fn peer_discovery(
    tls_connector: TlsConnector,
    config: NetworkConfig,
    network: Network,
    peers: Arc<Mutex<HashMap<SocketAddr, Peer>>>,
) {
    loop {
        if peers.lock().await.len() < config.target_peers
            && !connect_dns(&tls_connector, &config, &network, &peers).await
        {
            warn!("Insufficient peers");
        }
        sleep(Duration::from_secs(5)).await;
    }
}

#[instrument(skip(tls_connector, config, network, peers))]
async fn connect_dns(
    tls_connector: &TlsConnector,
    config: &NetworkConfig,
    network: &Network,
    peers: &Arc<Mutex<HashMap<SocketAddr, Peer>>>,
) -> bool {
    for host in &network.dns_introducers {
        if peers.lock().await.len() >= config.target_peers {
            return true;
        }

        let addrs = match lookup_host(format!("{host}:80")).await {
            Ok(addrs) => addrs,
            Err(error) => {
                warn!("Failed to lookup DNS introducer {host}: {error}");
                continue;
            }
        };

        let mut addrs: Vec<SocketAddr> = addrs.collect();
        addrs.as_mut_slice().shuffle(&mut thread_rng());

        println!("{}", addrs.len());

        for chunk in addrs.chunks(10) {
            connect_peers(
                tls_connector,
                config,
                peers,
                chunk
                    .iter()
                    .map(|addr| SocketAddr::new(addr.ip(), network.default_port))
                    .collect(),
            )
            .await;
        }
    }

    peers.lock().await.len() >= config.target_peers
}

#[instrument(skip(tls_connector, config, peers))]
async fn connect_peers(
    tls_connector: &TlsConnector,
    config: &NetworkConfig,
    peers: &Arc<Mutex<HashMap<SocketAddr, Peer>>>,
    socket_addrs: Vec<SocketAddr>,
) {
    let mut futures = FuturesUnordered::new();

    for socket_addr in socket_addrs {
        let network_id = config.network_id.clone();
        let tls_connector = tls_connector.clone();
        futures.push(async move {
            timeout(
                Duration::from_secs(3),
                connect_peer(network_id, tls_connector, socket_addr),
            )
            .await
        });
    }

    while let Some(result) = futures.next().await {
        let Ok(result) = result else {
            debug!("Connection timeout");
            continue;
        };

        if let Ok((peer, receiver)) = result {
            let mut peer_lock = peers.lock().await;

            if peer_lock.len() >= config.target_peers {
                debug!("Target peers already reached");
                break;
            }

            let socket_addr = peer.socket_addr();
            peer_lock.insert(socket_addr, peer);

            tokio::spawn(handle_peer(socket_addr, receiver, peers.clone()));

            info!("Successful connection");
        }
    }
}

#[instrument(skip(receiver, peers))]
async fn handle_peer(
    socket_addr: SocketAddr,
    mut receiver: mpsc::Receiver<Message>,
    peers: Arc<Mutex<HashMap<SocketAddr, Peer>>>,
) {
    while let Some(message) = receiver.recv().await {
        debug!("Received message {:?}", message.msg_type);
    }
    peers.lock().await.remove(&socket_addr);
    info!("Connection closed");
}
