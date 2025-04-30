use std::{io::Error, net::SocketAddr, time::Duration};

use futures_lite::StreamExt;
use futures_util::stream::FuturesUnordered;
use tracing::{info, warn};

pub async fn lookup_all(
    hosts: &[String],
    port: u16,
    timeout: Duration,
    batch_size: usize,
) -> Vec<SocketAddr> {
    let mut result = Vec::new();

    for batch in hosts.chunks(batch_size) {
        let mut futures = FuturesUnordered::new();

        for dns_introducer in batch {
            futures.push(async move {
                match tokio::time::timeout(timeout, lookup_host(dns_introducer, port)).await {
                    Ok(Ok(addrs)) => addrs,
                    Ok(Err(error)) => {
                        warn!("Failed to lookup DNS introducer {dns_introducer}: {error}");
                        Vec::new()
                    }
                    Err(_timeout) => {
                        warn!("Timeout looking up DNS introducer {dns_introducer}");
                        Vec::new()
                    }
                }
            });
        }

        while let Some(addrs) = futures.next().await {
            result.extend(addrs);
        }
    }

    result
}

async fn lookup_host(dns_introducer: &str, port: u16) -> Result<Vec<SocketAddr>, Error> {
    info!("Looking up DNS introducer {dns_introducer}");
    let mut result = Vec::new();
    for addr in tokio::net::lookup_host(format!("{dns_introducer}:80")).await? {
        result.push(SocketAddr::new(addr.ip(), port));
    }
    Ok(result)
}
