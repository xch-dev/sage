use std::{ops::Deref, sync::Arc};

use sage_config::{Network, NetworkConfig, WalletDefaults, MAINNET};
use tokio::sync::{mpsc::Sender, Mutex};

use crate::{PeerState, SyncCommand, SyncEvent, SyncOptions, Wallet};

#[derive(Debug, Clone)]
pub struct SyncState(Arc<SyncStateInner>);

impl Deref for SyncState {
    type Target = SyncStateInner;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub struct SyncStateInner {
    pub options: SyncOptions,
    pub network: Mutex<Network>,
    pub network_config: Mutex<NetworkConfig>,
    pub wallet: Mutex<Option<Wallet>>,
    pub wallet_defaults: Mutex<WalletDefaults>,
    pub wallet_config: Mutex<sage_config::Wallet>,
    pub peers: Mutex<PeerState>,
    pub commands: Sender<SyncCommand>,
    pub events: Sender<SyncEvent>,
}

impl SyncState {
    pub fn new(
        options: SyncOptions,
        commands: Sender<SyncCommand>,
        events: Sender<SyncEvent>,
    ) -> Self {
        Self(Arc::new(SyncStateInner {
            options,
            network: Mutex::new(MAINNET.clone()),
            network_config: Mutex::new(NetworkConfig::default()),
            wallet: Mutex::new(None),
            wallet_defaults: Mutex::new(WalletDefaults::default()),
            wallet_config: Mutex::new(sage_config::Wallet::default()),
            peers: Mutex::new(PeerState::default()),
            commands,
            events,
        }))
    }
}

impl SyncStateInner {
    pub async fn update_network(&self, network: Network) {
        *self.network.lock().await = network;
        self.commands
            .send(SyncCommand::Reset { remove_peers: true })
            .await
            .ok();
    }

    pub async fn update_network_config(&self, network_config: NetworkConfig) {
        *self.network_config.lock().await = network_config;
    }

    pub async fn update_wallet_defaults(&self, wallet_defaults: WalletDefaults) {
        *self.wallet_defaults.lock().await = wallet_defaults;
    }

    pub async fn login_wallet(&self, wallet: Wallet, wallet_config: sage_config::Wallet) {
        *self.wallet.lock().await = Some(wallet);
        *self.wallet_config.lock().await = wallet_config;
        self.commands
            .send(SyncCommand::Reset {
                remove_peers: false,
            })
            .await
            .ok();
    }

    pub async fn logout_wallet(&self) {
        *self.wallet.lock().await = None;
        *self.wallet_config.lock().await = sage_config::Wallet::default();
        self.commands
            .send(SyncCommand::Reset {
                remove_peers: false,
            })
            .await
            .ok();
    }

    pub async fn update_wallet_config(&self, wallet_config: sage_config::Wallet) {
        *self.wallet_config.lock().await = wallet_config;
    }
}
