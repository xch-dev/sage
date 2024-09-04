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
use sage_database::NftUriKind;
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

    let next_index = tx.derivation_index(false).await?;

    let receive_address = if next_index > 0 {
        let max = next_index - 1;
        let max_used = tx.max_used_derivation_index(false).await?;
        let mut index = max_used.map_or(0, |i| i + 1);
        if index > max {
            index = max;
        }
        let puzzle_hash = tx.p2_puzzle_hash(index, false).await?;

        Some(encode_address(puzzle_hash.to_bytes(), state.prefix())?)
    } else {
        None
    };

    tx.commit().await?;

    Ok(SyncStatus {
        balance: Amount::from_mojos(balance, state.unit().decimals),
        unit: state.unit().clone(),
        total_coins,
        synced_coins,
        receive_address: receive_address.unwrap_or_default(),
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

    let mut records = Vec::new();

    let mut tx = wallet.db.tx().await?;

    for nft in tx.nfts().await? {
        let uris = tx.nft_uris(nft.launcher_id).await?;
        let mut data_uris = Vec::new();
        let mut metadata_uris = Vec::new();
        let mut license_uris = Vec::new();

        for uri in uris {
            match uri.kind {
                NftUriKind::Data => data_uris.push(uri.uri),
                NftUriKind::Metadata => metadata_uris.push(uri.uri),
                NftUriKind::License => license_uris.push(uri.uri),
            }
        }

        records.push(NftRecord {
            encoded_id: encode_address(nft.launcher_id.to_bytes(), "nft")?,
            launcher_id: hex::encode(nft.launcher_id),
            encoded_owner_did: nft
                .current_owner
                .map(|owner| encode_address(owner.to_bytes(), "did:chia:"))
                .transpose()?,
            owner_did: nft.current_owner.map(hex::encode),
            coin_id: hex::encode(nft.coin_id),
            address: encode_address(nft.p2_puzzle_hash.to_bytes(), state.prefix())?,
            royalty_address: encode_address(nft.royalty_puzzle_hash.to_bytes(), state.prefix())?,
            royalty_percent: (BigDecimal::from(nft.royalty_ten_thousandths)
                / BigDecimal::from(100))
            .to_string(),
            data_uris,
            data_hash: nft.data_hash.map(hex::encode),
            metadata_uris,
            metadata_json: nft.metadata_json,
            metadata_hash: nft.metadata_hash.map(hex::encode),
            license_uris,
            license_hash: nft.license_hash.map(hex::encode),
            edition_number: nft.edition_number,
            edition_total: nft.edition_total,
        });
    }

    tx.commit().await?;

    Ok(records)
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

    let Some(master_pk) = state.keychain.extract_public_key(fingerprint)? else {
        return Ok(None);
    };

    Ok(Some(WalletInfo {
        name,
        fingerprint,
        public_key: hex::encode(master_pk.to_bytes()),
        kind: if state.keychain.has_secret_key(fingerprint) {
            WalletKind::Hot
        } else {
            WalletKind::Cold
        },
    }))
}

#[command]
#[specta]
pub async fn get_wallet_secrets(
    state: State<'_, AppState>,
    fingerprint: u32,
) -> Result<Option<WalletSecrets>> {
    let state = state.lock().await;

    let (mnemonic, Some(secret_key)) = state.keychain.extract_secrets(fingerprint, b"")? else {
        return Ok(None);
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
        state.keychain.add_mnemonic(&mnemonic, b"")?
    } else {
        let seed = mnemonic.to_seed("");
        let master_sk = SecretKey::from_seed(&seed);
        let master_pk = master_sk.public_key();
        state.keychain.add_public_key(&master_pk)?
    };
    state.save_keychain()?;

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
            state.keychain.add_public_key(&master_pk)?
        } else if let Ok(master_sk) = bytes.try_into() {
            let master_sk = SecretKey::from_bytes(&master_sk)?;
            state.keychain.add_secret_key(&master_sk, b"")?
        } else {
            return Err(Error::invalid_key("Must be 32 or 48 bytes"));
        }
    } else {
        let mnemonic = Mnemonic::from_str(&key)?;
        state.keychain.add_mnemonic(&mnemonic, b"")?
    };
    state.save_keychain()?;

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
