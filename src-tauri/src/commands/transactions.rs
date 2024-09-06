use std::collections::HashMap;

use chia::{
    bls::{
        master_to_wallet_unhardened_intermediate, sign, DerivableKey, PublicKey, SecretKey,
        Signature,
    },
    protocol::{Bytes32, CoinSpend, SpendBundle},
    puzzles::DeriveSynthetic,
};
use chia_wallet_sdk::{
    decode_address, select_coins, Conditions, Peer, RequiredSignature, SpendContext, StandardLayer,
    MAINNET_CONSTANTS, TESTNET11_CONSTANTS,
};
use clvmr::Allocator;
use sage_api::Amount;
use sage_wallet::Wallet;
use specta::specta;
use tauri::{command, State};
use tokio::sync::MutexGuard;

use crate::{
    app_state::{AppState, AppStateInner},
    error::{Error, Result},
    utils::fetch_puzzle_hash,
};

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

    let Some(amount) = amount.to_mojos(state.unit().decimals) else {
        return Err(Error::invalid_amount(&amount));
    };

    let Some(fee) = fee.to_mojos(state.unit().decimals) else {
        return Err(Error::invalid_amount(&fee));
    };

    let (puzzle_hash, prefix) = decode_address(&address)?;
    if prefix != state.prefix() {
        return Err(Error::invalid_prefix(&prefix));
    }

    let total_amount = amount as u128 + fee as u128;

    let mut tx = wallet.db.tx().await?;
    let mut keys = HashMap::new();

    let coins = tx.unspent_p2_coins().await?;

    for coin in &coins {
        let key = tx.indexed_synthetic_key(coin.puzzle_hash).await?;
        keys.insert(coin.puzzle_hash, key);
    }

    let Some(change_puzzle_hash) = fetch_puzzle_hash(&mut tx).await? else {
        return Err(Error::no_change_address());
    };

    tx.commit().await?;

    let coins = select_coins(coins, total_amount).map_err(|_| Error::insufficient_funds())?;
    let selected_amount = coins.iter().fold(0, |acc, coin| acc + coin.amount as u128);
    let change_amount: u64 = (selected_amount - total_amount)
        .try_into()
        .expect("Invalid change");

    let mut ctx = SpendContext::new();

    let origin_coin_id = coins[0].coin_id();

    for (i, &coin) in coins.iter().enumerate() {
        let pk = keys[&coin.puzzle_hash].1;
        let p2 = StandardLayer::new(pk);

        let conditions = if i == 0 {
            let mut conditions =
                Conditions::new().create_coin(puzzle_hash.into(), amount, Vec::new());

            if fee > 0 {
                conditions = conditions.reserve_fee(fee);
            }

            if change_amount > 0 {
                conditions = conditions.create_coin(change_puzzle_hash, change_amount, Vec::new());
            }

            conditions
        } else {
            Conditions::new().assert_concurrent_spend(origin_coin_id)
        };

        p2.spend(&mut ctx, coin, conditions)?;
    }

    let coin_spends = ctx.take();
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

    let Some(fee) = fee.to_mojos(state.unit().decimals) else {
        return Err(Error::invalid_amount(&fee));
    };

    let coin_ids = coin_ids
        .iter()
        .map(|coin_id| Ok(hex::decode(coin_id)?.try_into()?))
        .collect::<Result<Vec<Bytes32>>>()?;

    let mut tx = wallet.db.tx().await?;

    let mut coins = Vec::new();
    let mut total_amount = 0;

    for coin_id in coin_ids {
        let Some(coin_state) = tx.coin_state(coin_id).await? else {
            return Err(Error::unknown_coin_id());
        };
        coins.push(coin_state.coin);
        total_amount += coin_state.coin.amount as u128;
    }

    let mut keys = HashMap::new();

    for coin in &coins {
        let key = tx.indexed_synthetic_key(coin.puzzle_hash).await?;
        keys.insert(coin.puzzle_hash, key);
    }

    let Some(output_puzzle_hash) = fetch_puzzle_hash(&mut tx).await? else {
        return Err(Error::no_change_address());
    };

    tx.commit().await?;

    if fee as u128 > total_amount {
        return Err(Error::insufficient_coin_total());
    }

    let mut ctx = SpendContext::new();

    let origin_coin_id = coins[0].coin_id();

    for (i, &coin) in coins.iter().enumerate() {
        let pk = keys[&coin.puzzle_hash].1;
        let p2 = StandardLayer::new(pk);

        let conditions = if i == 0 {
            let mut conditions = Conditions::new();

            if fee > 0 {
                conditions = conditions.reserve_fee(fee);
            }

            if fee as u128 != total_amount {
                conditions = conditions.create_coin(
                    output_puzzle_hash,
                    (total_amount - fee as u128).try_into()?,
                    Vec::new(),
                );
            }

            conditions
        } else {
            Conditions::new().assert_concurrent_spend(origin_coin_id)
        };

        p2.spend(&mut ctx, coin, conditions)?;
    }

    let coin_spends = ctx.take();
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
        let ack = peer.send_transaction(spend_bundle.clone()).await?;

        log::info!(
            "Transaction sent to {} with ack {:?}",
            peer.socket_addr(),
            ack
        );
    }

    Ok(())
}
