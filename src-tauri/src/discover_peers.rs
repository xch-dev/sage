use std::{
    net::{IpAddr, SocketAddr},
    time::Duration,
};

use chia::protocol::Message;
use chia_wallet_sdk::{connect_peer, Client};
use futures_lite::StreamExt;
use futures_util::stream::FuturesUnordered;
use rand::{seq::SliceRandom, thread_rng};
use tokio::{
    sync::mpsc,
    time::{sleep, timeout},
};
use tracing::{debug, info, instrument};

use crate::config::{NetworkConfig, PeerMode};

#[instrument(skip(client, network_config))]
pub async fn update_peers(client: Client, network_config: NetworkConfig) {
    loop {
        if network_config.peer_mode == PeerMode::Automatic {
            discover_peers(&client, &network_config).await;
        }
        sleep(Duration::from_secs(5)).await;
    }
}

#[instrument(skip(client, network_config))]
async fn discover_peers(client: &Client, network_config: &NetworkConfig) {
    let peer_count = client.lock().await.peers().count();

    if peer_count >= network_config.target_peers {
        return;
    }

    let mut addrs = client
        .network()
        .lookup_all(Duration::from_secs(5), 10)
        .await;

    addrs.as_mut_slice().shuffle(&mut thread_rng());

    for chunk in addrs.chunks(10) {
        connect_peers(client, network_config, chunk).await;
    }
}

#[instrument(skip(client, network_config))]
async fn connect_peers(
    client: &Client,
    network_config: &NetworkConfig,
    socket_addrs: &[SocketAddr],
) {
    let mut futures = FuturesUnordered::new();

    for &socket_addr in socket_addrs {
        futures.push(async move {
            (
                socket_addr,
                timeout(Duration::from_secs(3), client.connect(socket_addr)).await,
            )
        });
    }

    while let Some((socket_addr, result)) = futures.next().await {
        let Ok(result) = result else {
            debug!("Connection timeout for {socket_addr}");
            continue;
        };

        let result = match result {
            Ok(result) => result,
            Err(error) => {
                debug!("Connection error for {socket_addr}: {error:?}");
                continue;
            }
        };

        let mut client = client.lock().await;

        if client.peers().count() >= network_config.target_peers {
            debug!("Target peers already reached, disconnecting {socket_addr}");
            break;
        }

        if !sync_manager.add_peer(peer) {
            continue;
        }

        tokio::spawn(handle_peer(ip_addr, receiver, ctx.sync_manager.clone()));

        info!("Successful connection");
    }
}

#[instrument(skip(receiver, client))]
async fn handle_peer(ip_addr: IpAddr, mut receiver: mpsc::Receiver<Message>, client: Client) {
    while let Some(message) = receiver.recv().await {
        debug!("Received message {:?}", message.msg_type);
    }

    client.lock().await.disconnect(&ip_addr);

    info!("Connection closed");
}
