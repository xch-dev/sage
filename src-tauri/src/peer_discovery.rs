use std::{collections::HashMap, net::SocketAddr, sync::Arc, time::Duration};

use futures_util::{stream::FuturesUnordered, StreamExt};
use native_tls::TlsConnector;
use rand::{seq::SliceRandom, thread_rng};
use sage_client::{connect_peer, Network, Peer};
use tokio::{
    net::lookup_host,
    sync::Mutex,
    time::{sleep, timeout},
};

pub async fn peer_discovery(
    tls_connector: TlsConnector,
    network_id: String,
    network: Network,
    target_peers: usize,
    peers: Arc<Mutex<HashMap<SocketAddr, Peer>>>,
) {
    'discovery: loop {
        if peers.lock().await.len() >= target_peers {
            sleep(Duration::from_secs(5)).await;
            continue;
        }

        for host in &network.dns_introducers {
            println!("Discovering peers from {}", host);

            if peers.lock().await.len() >= target_peers {
                continue 'discovery;
            }

            let addrs = match lookup_host(format!("{host}:80")).await {
                Ok(addrs) => addrs,
                Err(e) => {
                    println!("Failed to lookup host {}: {}", host, e);
                    continue;
                }
            };

            let mut addrs: Vec<SocketAddr> = addrs.collect();
            addrs.as_mut_slice().shuffle(&mut thread_rng());

            for chunk in addrs.chunks(10) {
                connect_peers(
                    tls_connector.clone(),
                    network_id.clone(),
                    target_peers,
                    peers.clone(),
                    chunk
                        .iter()
                        .map(|addr| SocketAddr::new(addr.ip(), network.default_port))
                        .collect(),
                )
                .await;
            }
        }

        sleep(Duration::from_secs(5)).await;
    }
}

async fn connect_peers(
    tls_connector: TlsConnector,
    network_id: String,
    target_peers: usize,
    peers: Arc<Mutex<HashMap<SocketAddr, Peer>>>,
    socket_addrs: Vec<SocketAddr>,
) {
    let mut futures = FuturesUnordered::new();

    for socket_addr in socket_addrs {
        let network_id = network_id.clone();
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
            println!("Timeout");
            continue;
        };

        if let Ok((peer, mut receiver)) = result {
            let peers_clone = peers.clone();

            let mut peers = peers.lock().await;

            if peers.len() >= target_peers {
                break;
            }

            let socket_addr = peer.socket_addr();
            peers.insert(socket_addr, peer);

            tokio::spawn(async move {
                while let Some(_message) = receiver.recv().await {
                    // TODO: Handle messages
                }
                peers_clone.lock().await.remove(&socket_addr);
            });
        }
    }
}
