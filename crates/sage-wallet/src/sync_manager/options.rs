use std::time::Duration;

#[derive(Debug, Clone, Copy)]
pub struct SyncOptions {
    pub target_peers: usize,
    pub discover_peers: bool,
    pub dns_batch_size: usize,
    pub connection_batch_size: usize,
    pub max_peer_age_seconds: u64,
    pub timeouts: Timeouts,
    pub testing: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct Timeouts {
    pub sync_delay: Duration,
    pub cat_delay: Duration,
    pub nft_uri_delay: Duration,
    pub puzzle_delay: Duration,
    pub transaction_delay: Duration,
    pub offer_delay: Duration,
    pub blocktime_delay: Duration,
    pub connection: Duration,
    pub initial_peak: Duration,
    pub remove_subscription: Duration,
    pub request_peers: Duration,
    pub dns: Duration,
    pub introducer: Duration,
}

impl Default for Timeouts {
    fn default() -> Self {
        Self {
            sync_delay: Duration::from_secs(1),
            cat_delay: Duration::from_secs(1),
            nft_uri_delay: Duration::from_millis(500),
            puzzle_delay: Duration::from_secs(1),
            transaction_delay: Duration::from_secs(1),
            offer_delay: Duration::from_secs(5),
            blocktime_delay: Duration::from_secs(1),
            connection: Duration::from_secs(3),
            initial_peak: Duration::from_secs(2),
            remove_subscription: Duration::from_secs(3),
            request_peers: Duration::from_secs(3),
            dns: Duration::from_secs(3),
            introducer: Duration::from_secs(10),
        }
    }
}
