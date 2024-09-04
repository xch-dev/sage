use std::{net::IpAddr, str::FromStr};

use bigdecimal::BigDecimal;
use bip39::Mnemonic;
use chia::bls::{PublicKey, SecretKey};
use chia_wallet_sdk::{decode_address, encode_address};
use indexmap::IndexMap;
use itertools::Itertools;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use sage_api::{Amount, CatRecord, CoinRecord, DidRecord, NftRecord, SyncStatus};
use sage_config::{NetworkConfig, WalletConfig};
use sage_keychain::{decrypt, KeyData, SecretKeyData};
use specta::specta;
use tauri::{command, State};

use crate::{
    app_state::AppState,
    error::{Error, Result},
    models::{NetworkInfo, PeerInfo, WalletInfo, WalletKind, WalletSecrets},
};

#[command]
#[specta]
pub async fn initialize(state: State<'_, AppState>) -> Result<()> {
    state.lock().await.initialize().await
}

#[command]
#[specta]
pub async fn get_sync_status(state: State<'_, AppState>) -> Result<SyncStatus> {
    let state = state.lock().await;
    let wallet = state.wallet()?;

    let mut tx = wallet.db.tx().await?;

    let balance = tx.p2_balance().await?;
    let total_coins = tx.total_coin_count().await?;
    let synced_coins = tx.synced_coin_count().await?;

    let max = tx.derivation_index(false).await? - 1;
    let max_used = tx.max_used_derivation_index(false).await?;
    let mut index = max_used.map_or(0, |i| i + 1);
    if index > max {
        index = max;
    }
    let p2_puzzle_hash = tx.p2_puzzle_hash(index, false).await?;

    tx.commit().await?;

    Ok(SyncStatus {
        balance: Amount::from_mojos(balance, state.unit().decimals),
        unit: state.unit().clone(),
        total_coins,
        synced_coins,
        receive_address: encode_address(p2_puzzle_hash.to_bytes(), state.prefix())?,
    })
}

#[command]
#[specta]
pub async fn get_coins(state: State<'_, AppState>) -> Result<Vec<CoinRecord>> {
    let state = state.lock().await;
    let wallet = state.wallet()?;
    let coin_states = wallet.db.p2_coin_states().await?;

    coin_states
        .into_iter()
        .map(|cs| {
            Ok(CoinRecord {
                coin_id: hex::encode(cs.coin.coin_id()),
                address: encode_address(cs.coin.puzzle_hash.to_bytes(), state.prefix())?,
                amount: Amount::from_mojos(cs.coin.amount as u128, state.unit().decimals),
                created_height: cs.created_height,
                spent_height: cs.spent_height,
            })
        })
        .collect()
}

#[command]
#[specta]
pub async fn get_cats(state: State<'_, AppState>) -> Result<Vec<CatRecord>> {
    let state = state.lock().await;
    let wallet = state.wallet()?;
    let cats = wallet.db.cats().await?;

    cats.into_iter()
        .map(|cat| {
            Ok(CatRecord {
                asset_id: hex::encode(cat.asset_id),
                name: cat.name,
                description: cat.description,
                ticker: cat.ticker,
                precision: cat.precision,
                icon_url: cat.icon_url,
            })
        })
        .collect()
}

#[command]
#[specta]
pub async fn get_dids(state: State<'_, AppState>) -> Result<Vec<DidRecord>> {
    let state = state.lock().await;
    let wallet = state.wallet()?;

    wallet
        .db
        .did_coins()
        .await?
        .into_iter()
        .map(|did| {
            Ok(DidRecord {
                encoded_id: encode_address(did.info.launcher_id.to_bytes(), "did:chia:")?,
                launcher_id: hex::encode(did.info.launcher_id),
                coin_id: hex::encode(did.coin.coin_id()),
                address: encode_address(did.info.p2_puzzle_hash.to_bytes(), state.prefix())?,
            })
        })
        .collect()
}

#[command]
#[specta]
pub async fn get_nfts(state: State<'_, AppState>) -> Result<Vec<NftRecord>> {
    let state = state.lock().await;
    let wallet = state.wallet()?;

    wallet
        .db
        .nft_coins()
        .await?
        .into_iter()
        .map(|nft| {
            Ok(NftRecord {
                encoded_id: encode_address(nft.info.launcher_id.to_bytes(), "nft")?,
                launcher_id: hex::encode(nft.info.launcher_id),
                coin_id: hex::encode(nft.coin.coin_id()),
                address: encode_address(nft.info.p2_puzzle_hash.to_bytes(), state.prefix())?,
                royalty_address: encode_address(
                    nft.info.royalty_puzzle_hash.to_bytes(),
                    state.prefix(),
                )?,
                royalty_percent: (BigDecimal::from(nft.info.royalty_ten_thousandths)
                    / BigDecimal::from(100))
                .to_string(),
            })
        })
        .collect()
}

#[command]
#[specta]
pub async fn validate_address(state: State<'_, AppState>, address: String) -> Result<bool> {
    let state = state.lock().await;
    let Some((_puzzle_hash, prefix)) = decode_address(&address).ok() else {
        return Ok(false);
    };
    Ok(prefix == state.prefix())
}

#[command]
#[specta]
pub async fn peer_list(state: State<'_, AppState>) -> Result<Vec<PeerInfo>> {
    let state = state.lock().await;
    let peer_state = state.peer_state.lock().await;

    Ok(peer_state
        .peers()
        .sorted_by_key(|peer| peer.socket_addr().ip())
        .map(|peer| PeerInfo {
            ip_addr: peer.socket_addr().ip().to_string(),
            port: peer.socket_addr().port(),
            trusted: false,
        })
        .collect())
}

#[command]
#[specta]
pub async fn remove_peer(state: State<'_, AppState>, ip_addr: IpAddr, ban: bool) -> Result<()> {
    let state = state.lock().await;
    let mut peer_state = state.peer_state.lock().await;

    if ban {
        peer_state.ban(ip_addr);
    } else {
        peer_state.remove_peer(ip_addr);
    }

    Ok(())
}

#[command]
#[specta]
pub async fn network_list(state: State<'_, AppState>) -> Result<IndexMap<String, NetworkInfo>> {
    let state = state.lock().await;

    let mut networks = IndexMap::new();

    for (network_id, network) in &state.networks {
        let info = NetworkInfo {
            default_port: network.default_port,
            genesis_challenge: hex::encode(network.genesis_challenge),
            agg_sig_me: network.agg_sig_me.map(hex::encode),
            dns_introducers: network.dns_introducers.clone(),
        };
        networks.insert(network_id.clone(), info);
    }

    Ok(networks)
}

#[command]
#[specta]
pub async fn network_config(state: State<'_, AppState>) -> Result<NetworkConfig> {
    let state = state.lock().await;
    Ok(state.config.network.clone())
}

#[command]
#[specta]
pub async fn set_discover_peers(state: State<'_, AppState>, discover_peers: bool) -> Result<()> {
    let mut state = state.lock().await;

    if state.config.network.discover_peers != discover_peers {
        state.config.network.discover_peers = discover_peers;
        state.save_config()?;
        state.reset_sync_task(false)?;
    }

    Ok(())
}

#[command]
#[specta]
pub async fn set_target_peers(state: State<'_, AppState>, target_peers: u32) -> Result<()> {
    let mut state = state.lock().await;

    state.config.network.target_peers = target_peers;
    state.save_config()?;
    state.reset_sync_task(false)?;

    Ok(())
}

#[command]
#[specta]
pub async fn set_network_id(state: State<'_, AppState>, network_id: String) -> Result<()> {
    let mut state = state.lock().await;

    state.config.network.network_id = network_id;
    state.save_config()?;
    state.reset_sync_task(true)?;
    state.switch_wallet().await?;

    Ok(())
}

#[command]
#[specta]
pub async fn wallet_config(state: State<'_, AppState>, fingerprint: u32) -> Result<WalletConfig> {
    let state = state.lock().await;
    state.try_wallet_config(fingerprint).cloned()
}

#[command]
#[specta]
pub async fn set_derive_automatically(
    state: State<'_, AppState>,
    fingerprint: u32,
    derive_automatically: bool,
) -> Result<()> {
    let mut state = state.lock().await;

    let config = state.try_wallet_config_mut(fingerprint)?;

    if config.derive_automatically != derive_automatically {
        config.derive_automatically = derive_automatically;
        state.save_config()?;
    }

    Ok(())
}

#[command]
#[specta]
pub async fn set_derivation_batch_size(
    state: State<'_, AppState>,
    fingerprint: u32,
    derivation_batch_size: u32,
) -> Result<()> {
    let mut state = state.lock().await;

    let config = state.try_wallet_config_mut(fingerprint)?;
    config.derivation_batch_size = derivation_batch_size;
    state.save_config()?;

    // TODO: Only if needed.
    state.reset_sync_task(false)?;

    Ok(())
}

#[command]
#[specta]
pub async fn rename_wallet(
    state: State<'_, AppState>,
    fingerprint: u32,
    name: String,
) -> Result<()> {
    let mut state = state.lock().await;

    let config = state.try_wallet_config_mut(fingerprint)?;
    config.name = name;
    state.save_config()?;

    Ok(())
}

#[command]
#[specta]
pub async fn active_wallet(state: State<'_, AppState>) -> Result<Option<WalletInfo>> {
    let state = state.lock().await;

    let Some(fingerprint) = state.config.app.active_fingerprint else {
        return Ok(None);
    };

    let name = state
        .config
        .wallets
        .get(&fingerprint.to_string())
        .map_or_else(
            || "Unnamed Wallet".to_string(),
            |config| config.name.clone(),
        );

    let Some(key) = state.keys.get(&fingerprint) else {
        return Ok(None);
    };

    let (master_pk, kind) = match key {
        KeyData::Public { master_pk } => (master_pk, WalletKind::Cold),
        KeyData::Secret { master_pk, .. } => (master_pk, WalletKind::Hot),
    };

    Ok(Some(WalletInfo {
        name,
        fingerprint,
        public_key: hex::encode(master_pk),
        kind,
    }))
}

#[command]
#[specta]
pub async fn get_wallet_secrets(
    state: State<'_, AppState>,
    fingerprint: u32,
) -> Result<Option<WalletSecrets>> {
    let state = state.lock().await;

    let Some(key) = state.keys.get(&fingerprint) else {
        return Ok(None);
    };

    let (mnemonic, secret_key) = match key {
        KeyData::Public { .. } => return Ok(None),
        KeyData::Secret {
            entropy, encrypted, ..
        } => {
            let data = decrypt::<SecretKeyData>(encrypted, b"")?;

            let mnemonic = if *entropy {
                Some(Mnemonic::from_entropy(&data.0)?)
            } else {
                None
            };

            let secret_key = if let Some(mnemonic) = mnemonic.as_ref() {
                SecretKey::from_seed(&mnemonic.to_seed(""))
            } else {
                SecretKey::from_bytes(&data.0.try_into().expect("invalid length"))?
            };

            (mnemonic, secret_key)
        }
    };

    Ok(Some(WalletSecrets {
        mnemonic: mnemonic.map(|m| m.to_string()),
        secret_key: hex::encode(secret_key.to_bytes()),
    }))
}

#[command]
#[specta]
pub async fn wallet_list(state: State<'_, AppState>) -> Result<Vec<WalletInfo>> {
    let state = state.lock().await;
    state.wallets()
}

#[command]
#[specta]
pub async fn login_wallet(state: State<'_, AppState>, fingerprint: u32) -> Result<()> {
    let mut state = state.lock().await;
    state.config.app.active_fingerprint = Some(fingerprint);
    state.save_config()?;
    state.switch_wallet().await
}

#[command]
#[specta]
pub async fn logout_wallet(state: State<'_, AppState>) -> Result<()> {
    let mut state = state.lock().await;
    state.config.app.active_fingerprint = None;
    state.save_config()?;
    state.switch_wallet().await
}

#[command]
#[specta]
pub fn generate_mnemonic(use_24_words: bool) -> Result<String> {
    let mut rng = ChaCha20Rng::from_entropy();
    let mnemonic = if use_24_words {
        let entropy: [u8; 32] = rng.gen();
        Mnemonic::from_entropy(&entropy)?
    } else {
        let entropy: [u8; 16] = rng.gen();
        Mnemonic::from_entropy(&entropy)?
    };
    Ok(mnemonic.to_string())
}

#[command]
#[specta]
pub async fn create_wallet(
    state: State<'_, AppState>,
    name: String,
    mnemonic: String,
    save_mnemonic: bool,
) -> Result<()> {
    let mut state = state.lock().await;
    let mnemonic = Mnemonic::from_str(&mnemonic)?;

    let fingerprint = if save_mnemonic {
        state.import_mnemonic(&mnemonic)?
    } else {
        let seed = mnemonic.to_seed("");
        let master_sk = SecretKey::from_seed(&seed);
        let master_pk = master_sk.public_key();
        state.import_public_key(&master_pk)?
    };

    let config = state.wallet_config_mut(fingerprint);
    config.name = name;
    state.config.app.active_fingerprint = Some(fingerprint);
    state.save_config()?;

    state.switch_wallet().await
}

#[command]
#[specta]
pub async fn import_wallet(state: State<'_, AppState>, name: String, key: String) -> Result<()> {
    let mut state = state.lock().await;

    let mut key_hex = key.as_str();

    if key_hex.starts_with("0x") || key_hex.starts_with("0X") {
        key_hex = &key_hex[2..];
    }

    let fingerprint = if let Ok(bytes) = hex::decode(key_hex) {
        if let Ok(master_pk) = bytes.clone().try_into() {
            let master_pk = PublicKey::from_bytes(&master_pk)?;
            state.import_public_key(&master_pk)?
        } else if let Ok(master_sk) = bytes.try_into() {
            let master_sk = SecretKey::from_bytes(&master_sk)?;
            state.import_secret_key(&master_sk)?
        } else {
            return Err(Error::invalid_key("Must be 32 or 48 bytes"));
        }
    } else {
        let mnemonic = Mnemonic::from_str(&key)?;
        state.import_mnemonic(&mnemonic)?
    };

    let config = state.wallet_config_mut(fingerprint);
    config.name = name;
    state.config.app.active_fingerprint = Some(fingerprint);
    state.save_config()?;

    state.switch_wallet().await
}

#[command]
#[specta]
pub async fn delete_wallet(state: State<'_, AppState>, fingerprint: u32) -> Result<()> {
    let mut state = state.lock().await;
    state.delete_wallet(fingerprint)?;
    Ok(())
}
