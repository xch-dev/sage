use std::{path::Path, sync::Arc, time::Duration};

use anyhow::{Context, Result, bail};
use bip39::Mnemonic;
use chia_wallet_sdk::{
    chia::{
        bls::master_to_wallet_unhardened,
        puzzle_types::{DeriveSynthetic, standard::StandardArgs},
    },
    prelude::*,
    test::PeerSimulator,
};
use sage::Sage;
use sage_api::{GetSyncStatus, ImportWallet, SyncEvent as ApiEvent};
use sage_wallet::{SyncCommand, SyncEvent};
use serde::Serialize;
use tokio::{
    sync::{Mutex, broadcast},
    time::timeout,
};
use tracing::info;

use crate::forward_events;

/// The wallets to start, with the seed byte their mnemonic is derived from.
const WALLETS: [(&str, u8); 2] = [("alice", 1), ("bob", 2)];

/// How many coins each wallet starts with, so that several spends can be pending at once.
const STARTING_COINS: usize = 10;

/// The amount of each starting coin, in mojos. This is 10 XCH.
const STARTING_COIN_AMOUNT: u64 = 10_000_000_000_000;

/// Starts every sandbox wallet, each with its data in a folder of `data_dir` named after it.
pub async fn start_wallets(
    data_dir: &Path,
    sim: &PeerSimulator,
    xchandles_url: &str,
) -> Result<Vec<SandboxWallet>> {
    let mut wallets = Vec::new();

    for (name, seed) in WALLETS {
        wallets.push(
            SandboxWallet::start(name, seed, &data_dir.join(name), sim, xchandles_url).await?,
        );
    }

    Ok(wallets)
}

/// A seeded wallet with its own Sage backend, connected to the shared simulator.
#[derive(Debug)]
pub struct SandboxWallet {
    pub info: WalletInfo,
    pub sage: Arc<Mutex<Sage>>,
    pub events: broadcast::Sender<ApiEvent>,
}

/// What agents and people need to know about a wallet. This is written to the ready file.
#[derive(Debug, Clone, Serialize)]
pub struct WalletInfo {
    pub name: String,
    pub fingerprint: u32,
    pub mnemonic: String,
    pub address: String,
}

impl SandboxWallet {
    /// Starts a backend in `dir`, funds a wallet derived from `seed`, and waits for it to sync.
    pub async fn start(
        name: &str,
        seed: u8,
        dir: &Path,
        sim: &PeerSimulator,
        xchandles_url: &str,
    ) -> Result<Self> {
        // A fixed mnemonic keeps fingerprints and addresses the same across runs.
        let mnemonic = Mnemonic::from_entropy(&[seed; 16])?;
        let master_sk = SecretKey::from_seed(&mnemonic.to_seed(""));
        let puzzle_hash = StandardArgs::curry_tree_hash(
            master_to_wallet_unhardened(&master_sk, 0)
                .public_key()
                .derive_synthetic(),
        );

        {
            let mut sim = sim.lock().await;
            sim.create_block();

            for _ in 0..STARTING_COINS {
                sim.new_coin(puzzle_hash.into(), STARTING_COIN_AMOUNT);
            }
        }

        let mut sage = Sage::new(dir, true);

        // Only the simulator should ever be connected.
        sage.config.network.target_peers = 1;
        sage.config.network.discover_peers = false;
        sage.config.network.default_network = "testnet11".to_string();

        let mut receiver = sage
            .initialize()
            .await
            .with_context(|| format!("failed to initialize the {name} wallet"))?;

        for network in &mut sage.network_list.networks {
            if let Some(mut config) = network.xchandles() {
                config.api_url = xchandles_url.to_string();
                network.xchandles = Some(config);
            }
        }

        let (peer, peer_receiver) = sim.connect_raw().await?;

        sage.command_sender
            .send(SyncCommand::AddPeer {
                peer,
                receiver: peer_receiver,
            })
            .await?;

        let fingerprint = sage
            .import_wallet(ImportWallet {
                name: capitalize(name),
                key: mnemonic.to_string(),
                derivation_index: 0,
                hardened: None,
                unhardened: None,
                save_secrets: true,
                login: true,
                emoji: None,
            })
            .await?
            .fingerprint;

        let subscribed = timeout(Duration::from_mins(1), async {
            while let Some(event) = receiver.recv().await {
                if matches!(event, SyncEvent::Subscribed) {
                    return true;
                }
            }

            false
        })
        .await;

        if !matches!(subscribed, Ok(true)) {
            bail!("the {name} wallet didn't finish syncing with the simulator");
        }

        let address = sage
            .get_sync_status(GetSyncStatus {})
            .await?
            .receive_address;

        info!("{name} is ready with fingerprint {fingerprint} and address {address}");

        let (events, _) = broadcast::channel(256);
        forward_events(receiver, events.clone());

        Ok(Self {
            info: WalletInfo {
                name: name.to_string(),
                fingerprint,
                mnemonic: mnemonic.to_string(),
                address,
            },
            sage: Arc::new(Mutex::new(sage)),
            events,
        })
    }
}

fn capitalize(name: &str) -> String {
    let mut chars = name.chars();

    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}
