use std::time::Duration;

#[derive(Debug, Clone, Copy)]
pub struct SyncOptions {
    pub target_peers: usize,
    pub discover_peers: bool,
    pub max_peers_for_dns: usize,
    pub dns_batch_size: usize,
    pub connection_batch_size: usize,
    pub max_peer_age_seconds: u64,
    pub sync_delay: Duration,
    pub connection_timeout: Duration,
    pub initial_peak_timeout: Duration,
    pub remove_subscription_timeout: Duration,
    pub request_peers_timeout: Duration,
    pub dns_timeout: Duration,
}
