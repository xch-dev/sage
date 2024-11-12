use std::time::Duration;

#[derive(Debug, Clone, Copy)]
pub struct SyncOptions {
    pub target_peers: usize,
    pub max_peers_for_dns: usize,
    pub dns_batch_size: usize,
    pub connection_batch_size: usize,
    pub max_peer_age_seconds: u64,
    pub sync_delay: Duration,
    pub timeouts: Timeouts,
}

#[derive(Debug, Clone, Copy)]
pub struct Timeouts {
    pub connection: Duration,
    pub initial_peak: Duration,
    pub remove_subscription: Duration,
    pub request_peers: Duration,
    pub dns: Duration,
}

impl Default for Timeouts {
    fn default() -> Self {
        Self {
            connection: Duration::from_secs(3),
            initial_peak: Duration::from_secs(2),
            remove_subscription: Duration::from_secs(3),
            request_peers: Duration::from_secs(3),
            dns: Duration::from_secs(3),
        }
    }
}
