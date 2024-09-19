use std::{collections::HashMap, time::Duration};

use chia::{
    bls::{
        master_to_wallet_unhardened_intermediate, sign, DerivableKey, PublicKey, SecretKey,
        Signature,
    },
    protocol::{Bytes32, CoinSpend, SpendBundle},
    puzzles::DeriveSynthetic,
};
use chia_wallet_sdk::{
    decode_address, Peer, RequiredSignature, MAINNET_CONSTANTS, TESTNET11_CONSTANTS,
};
use clvmr::Allocator;
use sage_api::Amount;
use sage_database::CatRow;
use sage_wallet::Wallet;
use specta::specta;
use tauri::{command, State};
use tokio::{sync::MutexGuard, time::timeout};

use crate::{
    app_state::{AppState, AppStateInner},
    error::{Error, Result},
};

#[command]
#[specta]
pub async fn validate_address(state: State<'_, AppState>, address: String) -> Result<bool> {
    let state = state.lock().await;
    let Some((_puzzle_hash, prefix)) = decode_address(&address).ok() else {
        return Ok(false);
    };
    Ok(prefix == state.network().address_prefix)
}

#[command]
#[specta]
pub async fn send(
    state: State<'_, AppState>,
    address: String,
    amount: Amount,
    fee: Amount,
) -> Result<()> {
    let state = state.lock().await;
    let wallet = state.wallet()?;

    if !state.keychain.has_secret_key(wallet.fingerprint) {
        return Err(Error::no_secret_key());
    }

    let (puzzle_hash, prefix) = decode_address(&address)?;
    if prefix != state.network().address_prefix {
        return Err(Error::invalid_prefix(&prefix));
    }

    let Some(amount) = amount.to_mojos(state.unit.decimals) else {
        return Err(Error::invalid_amount(&amount));
    };

    let Some(fee) = fee.to_mojos(state.unit.decimals) else {
        return Err(Error::invalid_amount(&fee));
    };

    let coin_spends = wallet
        .send_xch(puzzle_hash.into(), amount, fee, Vec::new(), false, true)
        .await?;

    transact(&state, &wallet, coin_spends).await?;

    Ok(())
}

#[command]
#[specta]
pub async fn combine(state: State<'_, AppState>, coin_ids: Vec<String>, fee: Amount) -> Result<()> {
    let state = state.lock().await;
    let wallet = state.wallet()?;

    if !state.keychain.has_secret_key(wallet.fingerprint) {
        return Err(Error::no_secret_key());
    }

    let Some(fee) = fee.to_mojos(state.unit.decimals) else {
        return Err(Error::invalid_amount(&fee));
    };

    let coin_ids = coin_ids
        .iter()
        .map(|coin_id| Ok(hex::decode(coin_id)?.try_into()?))
        .collect::<Result<Vec<Bytes32>>>()?;

    let mut tx = wallet.db.tx().await?;

    let mut coins = Vec::new();

    for coin_id in coin_ids {
        let Some(coin_state) = tx.coin_state(coin_id).await? else {
            return Err(Error::unknown_coin_id());
        };

        if coin_state.spent_height.is_some() {
            return Err(Error::already_spent(coin_id));
        }

        coins.push(coin_state.coin);
    }

    tx.commit().await?;

    let coin_spends = wallet
        .combine_xch(coins, fee, Vec::new(), false, true)
        .await?;

    transact(&state, &wallet, coin_spends).await?;

    Ok(())
}

#[command]
#[specta]
pub async fn split(
    state: State<'_, AppState>,
    coin_ids: Vec<String>,
    output_count: u32,
    fee: Amount,
) -> Result<()> {
    let state = state.lock().await;
    let wallet = state.wallet()?;

    if !state.keychain.has_secret_key(wallet.fingerprint) {
        return Err(Error::no_secret_key());
    }

    let Some(fee) = fee.to_mojos(state.unit.decimals) else {
        return Err(Error::invalid_amount(&fee));
    };

    let coin_ids = coin_ids
        .iter()
        .map(|coin_id| Ok(hex::decode(coin_id)?.try_into()?))
        .collect::<Result<Vec<Bytes32>>>()?;

    let mut tx = wallet.db.tx().await?;

    let mut coins = Vec::new();

    for coin_id in coin_ids {
        let Some(coin_state) = tx.coin_state(coin_id).await? else {
            return Err(Error::unknown_coin_id());
        };

        if coin_state.spent_height.is_some() {
            return Err(Error::already_spent(coin_id));
        }

        coins.push(coin_state.coin);
    }

    tx.commit().await?;

    let coin_spends = wallet
        .split_xch(&coins, output_count as usize, fee, Vec::new(), false, true)
        .await?;

    transact(&state, &wallet, coin_spends).await?;

    Ok(())
}

#[command]
#[specta]
pub async fn issue_cat(
    state: State<'_, AppState>,
    name: String,
    amount: Amount,
    fee: Amount,
) -> Result<()> {
    let state = state.lock().await;
    let wallet = state.wallet()?;

    if !state.keychain.has_secret_key(wallet.fingerprint) {
        return Err(Error::no_secret_key());
    }

    let Some(amount) = amount.to_mojos(3) else {
        return Err(Error::invalid_amount(&amount));
    };

    let Some(fee) = fee.to_mojos(state.unit.decimals) else {
        return Err(Error::invalid_amount(&fee));
    };

    let (coin_spends, asset_id) = wallet.issue_cat(amount, fee, None, false, true).await?;

    transact(&state, &wallet, coin_spends).await?;

    wallet
        .db
        .maybe_insert_cat(CatRow {
            asset_id,
            name: Some(name),
            ticker: None,
            description: None,
            icon_url: None,
            visible: true,
        })
        .await?;

    Ok(())
}

#[command]
#[specta]
pub async fn send_cat(
    state: State<'_, AppState>,
    asset_id: String,
    address: String,
    amount: Amount,
    fee: Amount,
) -> Result<()> {
    let state = state.lock().await;
    let wallet = state.wallet()?;

    if !state.keychain.has_secret_key(wallet.fingerprint) {
        return Err(Error::no_secret_key());
    }

    let asset_id: Bytes32 = hex::decode(asset_id)?.try_into()?;

    let (puzzle_hash, prefix) = decode_address(&address)?;
    if prefix != state.network().address_prefix {
        return Err(Error::invalid_prefix(&prefix));
    }

    let Some(amount) = amount.to_mojos(3) else {
        return Err(Error::invalid_amount(&amount));
    };

    let Some(fee) = fee.to_mojos(state.unit.decimals) else {
        return Err(Error::invalid_amount(&fee));
    };

    let coin_spends = wallet
        .send_cat(asset_id, puzzle_hash.into(), amount, fee, false, true)
        .await?;

    transact(&state, &wallet, coin_spends).await?;

    Ok(())
}

async fn transact(
    state: &MutexGuard<'_, AppStateInner>,
    wallet: &Wallet,
    coin_spends: Vec<CoinSpend>,
) -> Result<()> {
    let required_signatures = RequiredSignature::from_coin_spends(
        &mut Allocator::new(),
        &coin_spends,
        if state.config.network.network_id == "mainnet" {
            &MAINNET_CONSTANTS
        } else {
            &TESTNET11_CONSTANTS
        },
    )?;

    let mut indices = HashMap::new();

    for required in &required_signatures {
        let pk = required.public_key();
        let Some(index) = wallet.db.synthetic_key_index(pk).await? else {
            return Err(Error::unknown_public_key());
        };
        indices.insert(pk, index);
    }

    let (_mnemonic, Some(master_sk)) = state.keychain.extract_secrets(wallet.fingerprint, b"")?
    else {
        return Err(Error::no_secret_key());
    };

    let intermediate_sk = master_to_wallet_unhardened_intermediate(&master_sk);

    let secret_keys: HashMap<PublicKey, SecretKey> = indices
        .iter()
        .map(|(pk, index)| {
            (
                *pk,
                intermediate_sk.derive_unhardened(*index).derive_synthetic(),
            )
        })
        .collect();

    let mut aggregated_signature = Signature::default();

    for required in required_signatures {
        let sk = secret_keys[&required.public_key()].clone();
        aggregated_signature += &sign(&sk, required.final_message());
    }

    let spend_bundle = SpendBundle::new(coin_spends, aggregated_signature);

    let peers: Vec<Peer> = state
        .peer_state
        .lock()
        .await
        .peers()
        .map(|info| info.peer.clone())
        .collect();

    if peers.is_empty() {
        return Err(Error::no_peers());
    }

    log::info!(
        "Broadcasting transaction id {}: {:?}",
        spend_bundle.name(),
        spend_bundle
    );

    for peer in peers {
        let ack = match timeout(
            Duration::from_secs(3),
            peer.send_transaction(spend_bundle.clone()),
        )
        .await
        {
            Ok(Ok(ack)) => ack,
            Err(_timeout) => {
                log::warn!("Transaction timed out for {}", peer.socket_addr());
                continue;
            }
            Ok(Err(err)) => {
                log::warn!("Transaction failed for {}: {}", peer.socket_addr(), err);
                continue;
            }
        };

        log::info!(
            "Transaction sent to {} with ack {:?}",
            peer.socket_addr(),
            ack
        );
    }

    Ok(())
}
