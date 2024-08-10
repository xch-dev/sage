use std::{collections::HashMap, net::SocketAddr, sync::Arc, time::Duration};

use chia::protocol::Message;
use futures_util::{stream::FuturesUnordered, StreamExt};
use native_tls::TlsConnector;
use rand::{seq::SliceRandom, thread_rng};
use sage_client::{connect_peer, Network, Peer};
use tauri::{AppHandle, Emitter};
use tokio::{
    net::lookup_host,
    sync::{mpsc, Mutex},
    time::{sleep, timeout},
};
use tracing::{debug, info, instrument, warn};

use crate::config::NetworkConfig;

pub struct PeerContext {
    pub tls_connector: TlsConnector,
    pub config: NetworkConfig,
    pub network: Network,
    pub peers: Arc<Mutex<HashMap<SocketAddr, Peer>>>,
    pub app_handle: AppHandle,
}

#[instrument(skip(ctx))]
pub async fn peer_discovery(ctx: PeerContext) {
    loop {
        if ctx.peers.lock().await.len() < ctx.config.target_peers && !connect_dns(&ctx).await {
            warn!("Insufficient peers");
        }
        sleep(Duration::from_secs(5)).await;
    }
}

#[instrument(skip(ctx))]
async fn connect_dns(ctx: &PeerContext) -> bool {
    for host in &ctx.network.dns_introducers {
        if ctx.peers.lock().await.len() >= ctx.config.target_peers {
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

        debug!("Addreses: {addrs:?}");

        for chunk in addrs.chunks(5) {
            connect_peers(
                ctx,
                chunk
                    .iter()
                    .map(|addr| SocketAddr::new(addr.ip(), ctx.network.default_port))
                    .collect(),
            )
            .await;
        }
    }

    ctx.peers.lock().await.len() >= ctx.config.target_peers
}

#[instrument(skip(ctx))]
async fn connect_peers(ctx: &PeerContext, socket_addrs: Vec<SocketAddr>) {
    let mut futures = FuturesUnordered::new();

    for socket_addr in socket_addrs {
        let network_id = ctx.config.network_id.clone();
        let tls_connector = ctx.tls_connector.clone();
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
            let mut peer_lock = ctx.peers.lock().await;

            if peer_lock.len() >= ctx.config.target_peers {
                debug!("Target peers already reached");
                break;
            }

            let socket_addr = peer.socket_addr();
            peer_lock.insert(socket_addr, peer);

            tokio::spawn(handle_peer(
                socket_addr,
                receiver,
                ctx.peers.clone(),
                ctx.app_handle.clone(),
            ));

            info!("Successful connection");

            if let Err(error) = ctx.app_handle.emit("peer-update", ()) {
                warn!("Failed to emit peer update: {error}");
            }
        }
    }
}

#[instrument(skip(receiver, peers, app_handle))]
async fn handle_peer(
    socket_addr: SocketAddr,
    mut receiver: mpsc::Receiver<Message>,
    peers: Arc<Mutex<HashMap<SocketAddr, Peer>>>,
    app_handle: AppHandle,
) {
    while let Some(message) = receiver.recv().await {
        debug!("Received message {:?}", message.msg_type);
    }

    let mut peers = peers.lock().await;
    peers.remove(&socket_addr);

    if let Err(error) = app_handle.emit("peer-update", ()) {
        warn!("Failed to emit peer update: {error}");
    }

    info!("Connection closed");
}
