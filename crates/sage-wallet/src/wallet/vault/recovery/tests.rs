use std::{slice, time::Duration};

use chia_wallet_sdk::{driver::VaultVersion, prelude::*};
use sage_database::{VaultConfig, VaultLayout, VaultMember, VaultSigners, VaultState, VaultStatus};
use test_log::test;
use tokio::time::sleep;

use crate::{TestWallet, WalletError, sign_with_secret_keys, spend_start_recovery, vault_hashes};

const TIMELOCK: u64 = 60 * 60;

fn custody_key() -> SecretKey {
    SecretKey::from_seed(&[7; 32])
}

fn recovery_key() -> SecretKey {
    SecretKey::from_seed(&[8; 32])
}

fn new_custody_key() -> SecretKey {
    SecretKey::from_seed(&[9; 32])
}

fn config_with(custody: &SecretKey, version: VaultVersion) -> VaultConfig {
    VaultConfig::with_recovery(
        VaultSigners::new(vec![VaultMember::bls(custody.public_key())], 1),
        VaultSigners::new(vec![VaultMember::bls(recovery_key().public_key())], 1),
        TIMELOCK,
        version,
    )
}

fn launcher_id(vault: &TestWallet) -> Bytes32 {
    vault.wallet.vault_launcher_id().expect("a vault wallet")
}

/// Waits until the vault's latest coin is confirmed and unspent, and its status matches.
async fn wait_for_status(
    vault: &mut TestWallet,
    f: impl Fn(&VaultStatus) -> bool,
) -> anyhow::Result<VaultStatus> {
    let mut last = None;

    for _ in 0..100 {
        // Sync blocks once its event channel is full, so the events are drained while waiting.
        while vault.events.try_recv().is_ok() {}

        // The test database uses a shared cache, where a read transaction can deadlock with sync's
        // writes, so this takes the write lock up front instead.
        let mut tx = vault.wallet.db.tx().await?;
        let status = tx.vault_status(launcher_id(vault)).await?;
        tx.commit().await?;

        if let Some(status) = &status
            && status
                .coin
                .is_some_and(|coin| coin.created_height.is_some() && coin.spent_height.is_none())
            && f(status)
        {
            return Ok(status.clone());
        }

        last = status;
        sleep(Duration::from_millis(100)).await;
    }

    anyhow::bail!("timed out waiting for the vault, which is {last:?}");
}

async fn wait_for_custody(
    vault: &mut TestWallet,
    config: &VaultConfig,
) -> anyhow::Result<VaultStatus> {
    let custody_hash: Bytes32 = vault_hashes(config)?.custody_hash.into();

    wait_for_status(vault, |status| {
        status.state == VaultState::Active
            && status.coin.and_then(|coin| coin.custody_hash) == Some(custody_hash)
            && status.config.is_some()
    })
    .await
}

/// Waits for the vault to be in the recovery state, after a spend confirmed in a block with the
/// timestamp.
///
/// The peer simulator doesn't serve block headers, so the wallet never syncs block timestamps.
/// The timestamp of the block the recovery was confirmed in is stored as if it had, since the
/// countdown only uses block timestamps.
async fn wait_for_recovery(vault: &mut TestWallet, timestamp: u64) -> anyhow::Result<VaultStatus> {
    let status = wait_for_status(vault, |status| {
        matches!(status.state, VaultState::RecoveryPending { .. })
    })
    .await?;

    let height = status
        .coin
        .and_then(|coin| coin.created_height)
        .expect("the coin is confirmed");
    let header_hash = vault
        .sim
        .lock()
        .await
        .header_hash_of(height)
        .expect("the block exists");

    vault
        .wallet
        .db
        .insert_block(height, header_hash, Some(timestamp.try_into()?), false)
        .await?;

    wait_for_status(vault, |status| {
        matches!(
            status.state,
            VaultState::RecoveryPending {
                timelock_from: Some(_),
                ..
            }
        )
    })
    .await
}

/// Waits until the wallet's XCH balance is the expected one.
async fn wait_for_balance(wallet: &mut TestWallet, balance: u128) -> anyhow::Result<()> {
    for _ in 0..20 {
        if wallet.wallet.db.xch_balance().await? == balance {
            return Ok(());
        }

        wallet.wait_for_coins().await;
    }

    anyhow::bail!("timed out waiting for a balance of {balance}");
}

/// The timestamp of the block that the next transaction is confirmed in.
async fn next_timestamp(vault: &TestWallet) -> u64 {
    vault.sim.lock().await.next_timestamp()
}

/// Mints a vault from alice's wallet, and opens a vault wallet for it signed by the custody key,
/// with `balance` at its first address.
async fn mint(
    alice: &mut TestWallet,
    config: &VaultConfig,
    balance: u64,
) -> anyhow::Result<TestWallet> {
    let mint = alice.wallet.mint_vault(config, 0).await?;
    alice.transact(mint.coin_spends).await?;
    alice.wait_for_coins().await;

    let mut vault = alice.vault_wallet(mint.launcher_id, custody_key()).await?;
    wait_for_custody(&mut vault, config).await?;

    if balance > 0 {
        let coin_spends = alice
            .wallet
            .send_xch(vec![(vault.puzzle_hash, balance)], 0, vec![], None)
            .await?;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        wait_for_balance(&mut vault, balance.into()).await?;
    }

    Ok(vault)
}

/// Pushes coin spends signed by the keys, without the wallet's own key.
async fn push_signed(
    wallet: &TestWallet,
    coin_spends: Vec<CoinSpend>,
    keys: &[SecretKey],
) -> anyhow::Result<()> {
    let spend_bundle = sign_with_secret_keys(
        SpendBundle::new(coin_spends, Signature::default()),
        &wallet.agg_sig,
        keys,
        false,
    )?;
    wallet.push_bundle(spend_bundle).await
}

/// Starts a recovery to a new custody key with the recovery key, and waits for it.
async fn start_recovery(vault: &mut TestWallet, post_recovery: &VaultConfig) -> anyhow::Result<()> {
    let start = vault
        .wallet
        .start_vault_recovery(post_recovery, &[recovery_key().public_key()])
        .await?;
    let timestamp = next_timestamp(vault).await;
    push_signed(vault, start.coin_spends, slice::from_ref(&recovery_key())).await?;
    wait_for_recovery(vault, timestamp).await?;
    Ok(())
}

#[test(tokio::test)]
async fn test_vault_recovery_cancel() -> anyhow::Result<()> {
    for version in [VaultVersion::Legacy, VaultVersion::ForceSingletonRecreation] {
        let mut alice = TestWallet::new(10_000).await?;
        let config = config_with(&custody_key(), version);
        let post_recovery = config_with(&new_custody_key(), VaultVersion::ForceSingletonRecreation);
        let mut vault = mint(&mut alice, &config, 1000).await?;

        start_recovery(&mut vault, &post_recovery).await?;

        // The vault can't be spent normally, or recovered again, while recovery is pending.
        let result = vault
            .wallet
            .send_xch(vec![(alice.puzzle_hash, 100)], 0, vec![], None)
            .await;
        assert!(
            matches!(result, Err(WalletError::VaultRecoveryPending)),
            "{result:?}"
        );

        let pending = vault.wallet.pending_recovery().await?;
        assert!(pending.finish.is_some());
        assert_eq!(pending.pre_recovery.as_ref(), Some(&config));
        assert!(!pending.is_started_locally);

        // The custody cancels it, paying the fee from the vault's XCH.
        let coin_spends = vault.wallet.cancel_vault_recovery(100).await?;
        vault.transact(coin_spends).await?;

        let status = wait_for_custody(&mut vault, &config).await?;
        assert_eq!(
            status.config.map(|row| row.layout),
            Some(VaultLayout::Custody(config.clone()))
        );

        wait_for_balance(&mut vault, 900).await?;

        // The vault can be spent again.
        let coin_spends = vault
            .wallet
            .send_xch(vec![(alice.puzzle_hash, 100)], 0, vec![], None)
            .await?;
        vault.transact(coin_spends).await?;
    }

    Ok(())
}

#[test(tokio::test)]
async fn test_vault_recovery_finish() -> anyhow::Result<()> {
    for version in [VaultVersion::Legacy, VaultVersion::ForceSingletonRecreation] {
        let mut alice = TestWallet::new(10_000).await?;
        let config = config_with(&custody_key(), version);
        let post_recovery = config_with(&new_custody_key(), VaultVersion::ForceSingletonRecreation);
        let mut vault = mint(&mut alice, &config, 1000).await?;

        start_recovery(&mut vault, &post_recovery).await?;

        let pending = vault.wallet.pending_recovery().await?;
        let finishes_at = pending.finishes_at().expect("the timestamp is known");

        // It can't be finished before the timelock expires.
        let result = vault.wallet.finish_vault_recovery(finishes_at - 1).await;
        assert!(
            matches!(result, Err(WalletError::VaultRecoveryTimelock(Some(at))) if at == finishes_at),
            "{result:?}"
        );

        alice.sim.lock().await.pass_time(TIMELOCK);

        // Finishing needs no signature.
        let finish = vault.wallet.finish_vault_recovery(finishes_at).await?;
        assert_eq!(finish.post_recovery, post_recovery);
        vault
            .push_bundle(SpendBundle::new(finish.coin_spends, Signature::default()))
            .await?;

        wait_for_custody(&mut vault, &post_recovery).await?;

        // The old custody key can no longer sign for the vault.
        let coin_spends = vault
            .wallet
            .send_xch(vec![(alice.puzzle_hash, 100)], 0, vec![], None)
            .await?;
        assert!(vault.sign(coin_spends).await.is_err());

        // The new custody key can.
        let mut recovered = alice
            .vault_wallet(launcher_id(&vault), new_custody_key())
            .await?;
        wait_for_custody(&mut recovered, &post_recovery).await?;

        wait_for_balance(&mut recovered, 1000).await?;

        let coin_spends = recovered
            .wallet
            .send_xch(vec![(alice.puzzle_hash, 100)], 0, vec![], None)
            .await?;
        recovered.transact(coin_spends).await?;
    }

    Ok(())
}

#[test(tokio::test)]
async fn test_vault_rekey() -> anyhow::Result<()> {
    let mut alice = TestWallet::new(10_000).await?;

    // A legacy vault is rekeyed to a new custody key, and upgraded.
    let config = config_with(&custody_key(), VaultVersion::Legacy);
    let new_config = config_with(&new_custody_key(), VaultVersion::ForceSingletonRecreation);
    let mut vault = mint(&mut alice, &config, 1000).await?;

    // Rekeying to the same config does nothing.
    let result = vault.wallet.rekey_vault(&config, 0).await;
    assert!(
        matches!(result, Err(WalletError::VaultRekeyUnchanged)),
        "{result:?}"
    );

    let coin_spends = vault.wallet.rekey_vault(&new_config, 100).await?;
    vault.transact(coin_spends).await?;

    // The memo reveals the new config.
    let status = wait_for_custody(&mut vault, &new_config).await?;
    assert_eq!(
        status.config.map(|row| row.layout),
        Some(VaultLayout::Custody(new_config.clone()))
    );

    let mut rekeyed = alice
        .vault_wallet(launcher_id(&vault), new_custody_key())
        .await?;
    wait_for_custody(&mut rekeyed, &new_config).await?;

    wait_for_balance(&mut rekeyed, 900).await?;

    let coin_spends = rekeyed
        .wallet
        .send_xch(vec![(alice.puzzle_hash, 100)], 0, vec![], None)
        .await?;
    rekeyed.transact(coin_spends).await?;

    Ok(())
}

/// A recovery started elsewhere, without a memo, is still noticed from the spend that started it,
/// and can be cancelled, but not finished.
#[test(tokio::test)]
async fn test_vault_recovery_without_memo() -> anyhow::Result<()> {
    for version in [VaultVersion::Legacy, VaultVersion::ForceSingletonRecreation] {
        let mut alice = TestWallet::new(10_000).await?;
        let config = config_with(&custody_key(), version);
        let post_recovery = config_with(&new_custody_key(), VaultVersion::ForceSingletonRecreation);
        let mut vault = mint(&mut alice, &config, 0).await?;

        let (coin, _) = vault.wallet.active_vault().await?;
        let mut ctx = SpendContext::new();
        spend_start_recovery(
            &mut ctx,
            &coin,
            &config,
            &post_recovery,
            &[recovery_key().public_key()],
            false,
        )?;
        let timestamp = next_timestamp(&vault).await;
        push_signed(&vault, ctx.take(), slice::from_ref(&recovery_key())).await?;

        let status = wait_for_recovery(&mut vault, timestamp).await?;
        assert!(status.config.is_none());
        let recovery = status.recovery.expect("the recovery is recorded");
        assert!(!recovery.is_started_locally);
        assert_eq!(recovery.recovery_timelock, TIMELOCK);

        let pending = vault.wallet.pending_recovery().await?;
        assert!(pending.finish.is_none());

        let result = vault.wallet.finish_vault_recovery(u64::MAX).await;
        assert!(
            matches!(result, Err(WalletError::VaultRecoveryUnfinishable)),
            "{result:?}"
        );

        let coin_spends = vault.wallet.cancel_vault_recovery(0).await?;
        vault.transact(coin_spends).await?;
        wait_for_custody(&mut vault, &config).await?;
    }

    Ok(())
}

/// A recovery that this wallet started is remembered, so it isn't alerted about.
#[test(tokio::test)]
async fn test_vault_recovery_started_locally() -> anyhow::Result<()> {
    let mut alice = TestWallet::new(10_000).await?;
    let config = config_with(&custody_key(), VaultVersion::ForceSingletonRecreation);
    let post_recovery = config_with(&new_custody_key(), VaultVersion::ForceSingletonRecreation);
    let mut vault = mint(&mut alice, &config, 0).await?;

    let start = vault
        .wallet
        .start_vault_recovery(&post_recovery, &[recovery_key().public_key()])
        .await?;
    vault.wallet.record_started_recovery(start.recovery).await?;
    let timestamp = next_timestamp(&vault).await;
    push_signed(&vault, start.coin_spends, slice::from_ref(&recovery_key())).await?;

    let status = wait_for_recovery(&mut vault, timestamp).await?;
    assert!(
        status
            .recovery
            .expect("the recovery is recorded")
            .is_started_locally
    );
    assert!(vault.wallet.pending_recovery().await?.is_started_locally);

    Ok(())
}

/// A recovery or rekey can't be made while the vault has a pending spend.
#[test(tokio::test)]
async fn test_vault_recovery_refused_while_pending() -> anyhow::Result<()> {
    let mut alice = TestWallet::new(10_000).await?;
    let config = config_with(&custody_key(), VaultVersion::ForceSingletonRecreation);
    let post_recovery = config_with(&new_custody_key(), VaultVersion::ForceSingletonRecreation);
    let vault = mint(&mut alice, &config, 3000).await?;

    let coin_spends = vault
        .wallet
        .send_xch(vec![(alice.puzzle_hash, 1000)], 0, vec![], None)
        .await?;
    let spend_bundle = vault.sign(coin_spends).await?;

    // The transaction is recorded as pending, but the sync manager doesn't get to submit it.
    let peer = vault.state.lock().await.acquire_peer().expect("no peer");
    crate::insert_transaction(
        &vault.wallet.db,
        &peer,
        TESTNET11_CONSTANTS.genesis_challenge,
        spend_bundle.name(),
        crate::Transaction::from_coin_spends(spend_bundle.coin_spends)?,
        spend_bundle.aggregated_signature,
    )
    .await?;

    let result = vault
        .wallet
        .start_vault_recovery(&post_recovery, &[recovery_key().public_key()])
        .await;
    assert!(
        matches!(result, Err(WalletError::VaultSpendPending)),
        "{result:?}"
    );

    let result = vault.wallet.rekey_vault(&post_recovery, 0).await;
    assert!(
        matches!(result, Err(WalletError::VaultSpendPending)),
        "{result:?}"
    );

    Ok(())
}

/// A vault without recovery can't be recovered.
#[test(tokio::test)]
async fn test_vault_without_recovery_cant_recover() -> anyhow::Result<()> {
    let mut alice = TestWallet::new(10_000).await?;
    let config = VaultConfig::without_recovery(VaultSigners::new(
        vec![VaultMember::bls(custody_key().public_key())],
        1,
    ));
    let vault = mint(&mut alice, &config, 0).await?;

    let result = vault
        .wallet
        .start_vault_recovery(&config, &[recovery_key().public_key()])
        .await;
    assert!(
        matches!(result, Err(WalletError::VaultHasNoRecovery)),
        "{result:?}"
    );

    let result = vault.wallet.cancel_vault_recovery(0).await;
    assert!(
        matches!(result, Err(WalletError::VaultNoRecoveryPending)),
        "{result:?}"
    );

    Ok(())
}
