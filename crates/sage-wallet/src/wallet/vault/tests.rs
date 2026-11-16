use chia_wallet_sdk::{chia::puzzle_types::Memos, driver::VaultVersion, prelude::*};
use sage_database::{VaultConfig, VaultMember, VaultSigners, VaultState};
use test_log::test;

use crate::{
    MultiSendPayment, SyncEvent, TestWallet, WalletError, custody_mips_spend,
    parse_vault_spend_bundle, vault_hashes,
};

pub(super) const TIMELOCK: u64 = 60 * 60;

pub(super) fn custody_key() -> SecretKey {
    SecretKey::from_seed(&[7; 32])
}

pub(super) fn recovery_key() -> SecretKey {
    SecretKey::from_seed(&[8; 32])
}

pub(super) fn config(version: VaultVersion) -> VaultConfig {
    VaultConfig::with_recovery(
        VaultSigners::new(vec![VaultMember::bls(custody_key().public_key())], 1),
        VaultSigners::new(vec![VaultMember::bls(recovery_key().public_key())], 1),
        TIMELOCK,
        version,
    )
}

/// Waits until the vault wallet has synced the vault's current coin, and it's active.
pub(super) async fn wait_for_vault(vault: &mut TestWallet) -> anyhow::Result<()> {
    let launcher_id = vault.wallet.vault_launcher_id().expect("a vault wallet");

    loop {
        if let Some(status) = vault.wallet.db.vault_status(launcher_id).await?
            && status.state == VaultState::Active
            && status.config.is_some()
            && status.coin.is_some_and(|coin| coin.spent_height.is_none())
        {
            return Ok(());
        }

        vault
            .consume_until(|event| {
                matches!(
                    event,
                    SyncEvent::VaultStateChanged { .. } | SyncEvent::CoinsUpdated
                )
            })
            .await;
    }
}

/// Mints a vault from alice's wallet, and opens a vault wallet for it with `balance` at its first
/// address.
pub(super) async fn mint(
    alice: &mut TestWallet,
    config: &VaultConfig,
    balance: u64,
) -> anyhow::Result<TestWallet> {
    let mint = alice.wallet.mint_vault(config, 0).await?;
    alice.transact(mint.coin_spends).await?;
    alice.wait_for_coins().await;

    let mut vault = alice.vault_wallet(mint.launcher_id, custody_key()).await?;
    wait_for_vault(&mut vault).await?;

    if balance > 0 {
        let coin_spends = alice
            .wallet
            .send_xch(vec![(vault.puzzle_hash, balance)], 0, vec![], None)
            .await?;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;
        vault.wait_for_coins().await;
    }

    Ok(vault)
}

/// Pushes a vault spend and waits for it, and the recreated vault, to sync.
pub(super) async fn vault_transact(
    vault: &mut TestWallet,
    coin_spends: Vec<CoinSpend>,
) -> anyhow::Result<()> {
    vault.transact(coin_spends).await?;
    vault.wait_for_coins().await;
    wait_for_vault(vault).await
}

#[test(tokio::test)]
async fn test_mint_vault() -> anyhow::Result<()> {
    for version in [VaultVersion::Legacy, VaultVersion::ForceSingletonRecreation] {
        let mut alice = TestWallet::new(1000).await?;
        let config = config(version);

        let vault = mint(&mut alice, &config, 0).await?;
        let launcher_id = vault.wallet.vault_launcher_id().expect("a vault wallet");

        // The mint pays the singleton's mojo from alice's wallet.
        assert_eq!(alice.wallet.db.xch_balance().await?, 999);

        // The memo in the launcher solution decodes back to the config.
        let status = vault
            .wallet
            .db
            .vault_status(launcher_id)
            .await?
            .expect("the vault is tracked");
        let row = status.config.expect("the config is known");
        assert_eq!(row.custody_hash, vault_hashes(&config)?.custody_hash.into());
        assert_eq!(row.layout, sage_database::VaultLayout::Custody(config));
    }

    Ok(())
}

#[test(tokio::test)]
async fn test_mint_vault_with_fee() -> anyhow::Result<()> {
    let mut alice = TestWallet::new(1000).await?;

    let mint = alice
        .wallet
        .mint_vault(&config(VaultVersion::ForceSingletonRecreation), 100)
        .await?;
    alice.transact(mint.coin_spends).await?;
    alice.wait_for_coins().await;

    assert_eq!(alice.wallet.db.xch_balance().await?, 899);

    Ok(())
}

#[test(tokio::test)]
async fn test_vault_send_xch() -> anyhow::Result<()> {
    for version in [VaultVersion::Legacy, VaultVersion::ForceSingletonRecreation] {
        let mut alice = TestWallet::new(10_000).await?;
        let mut vault = mint(&mut alice, &config(version), 5000).await?;

        assert_eq!(vault.wallet.db.xch_balance().await?, 5000);

        let coin_spends = vault
            .wallet
            .send_xch(vec![(alice.puzzle_hash, 1000)], 100, vec![], None)
            .await?;
        vault_transact(&mut vault, coin_spends).await?;

        // The change goes back to the vault's first address.
        assert_eq!(vault.wallet.db.xch_balance().await?, 3900);
        let coins = vault.wallet.db.selectable_xch_coins().await?;
        assert_eq!(coins.len(), 1);
        assert_eq!(coins[0].puzzle_hash, vault.puzzle_hash);

        alice.wait_for_coins().await;
        assert_eq!(
            alice.wallet.db.xch_balance().await?,
            10_000 - 1 - 5000 + 1000
        );

        // The vault was recreated, so it can spend again.
        let coin_spends = vault
            .wallet
            .send_xch(vec![(alice.puzzle_hash, 900)], 0, vec![], None)
            .await?;
        vault_transact(&mut vault, coin_spends).await?;
        assert_eq!(vault.wallet.db.xch_balance().await?, 3000);
    }

    Ok(())
}

#[test(tokio::test)]
async fn test_vault_fee_only() -> anyhow::Result<()> {
    let mut alice = TestWallet::new(10_000).await?;
    let mut vault = mint(
        &mut alice,
        &config(VaultVersion::ForceSingletonRecreation),
        5000,
    )
    .await?;

    let coin_spends = vault.wallet.combine(vec![], 250).await?;
    vault_transact(&mut vault, coin_spends).await?;

    assert_eq!(vault.wallet.db.xch_balance().await?, 4750);

    Ok(())
}

#[test(tokio::test)]
async fn test_vault_send_cat() -> anyhow::Result<()> {
    let mut alice = TestWallet::new(10_000).await?;
    let mut vault = mint(
        &mut alice,
        &config(VaultVersion::ForceSingletonRecreation),
        1000,
    )
    .await?;

    let (coin_spends, asset_id) = alice.wallet.issue_cat(3000, 0, None).await?;
    alice.transact(coin_spends).await?;
    alice.wait_for_coins().await;

    let coin_spends = alice
        .wallet
        .send_cat(
            asset_id,
            vec![(vault.puzzle_hash, 2000)],
            0,
            true,
            vec![],
            None,
        )
        .await?;
    alice.transact(coin_spends).await?;
    alice.wait_for_coins().await;
    vault.wait_for_puzzles().await;

    assert_eq!(vault.wallet.db.cat_balance(asset_id).await?, 2000);

    let coin_spends = vault
        .wallet
        .send_cat(
            asset_id,
            vec![(alice.puzzle_hash, 500)],
            10,
            true,
            vec![],
            None,
        )
        .await?;
    vault_transact(&mut vault, coin_spends).await?;

    assert_eq!(vault.wallet.db.cat_balance(asset_id).await?, 1500);
    assert_eq!(vault.wallet.db.xch_balance().await?, 990);

    let cats = vault.wallet.db.selectable_cat_coins(asset_id).await?;
    assert_eq!(cats.len(), 1);
    assert_eq!(cats[0].info.p2_puzzle_hash, vault.puzzle_hash);

    alice.wait_for_puzzles().await;
    assert_eq!(alice.wallet.db.cat_balance(asset_id).await?, 1500);

    Ok(())
}

#[test(tokio::test)]
async fn test_vault_multi_send() -> anyhow::Result<()> {
    let mut alice = TestWallet::new(10_000).await?;
    let mut vault = mint(
        &mut alice,
        &config(VaultVersion::ForceSingletonRecreation),
        2000,
    )
    .await?;

    let (coin_spends, asset_id) = alice.wallet.issue_cat(1000, 0, None).await?;
    alice.transact(coin_spends).await?;
    alice.wait_for_coins().await;

    let coin_spends = alice
        .wallet
        .send_cat(
            asset_id,
            vec![(vault.puzzle_hash, 1000)],
            0,
            true,
            vec![],
            None,
        )
        .await?;
    alice.transact(coin_spends).await?;
    alice.wait_for_coins().await;
    vault.wait_for_puzzles().await;

    let bob = Bytes32::new([42; 32]);

    let coin_spends = vault
        .wallet
        .multi_send(
            vec![
                MultiSendPayment::xch(bob, 300),
                MultiSendPayment::xch(alice.puzzle_hash, 200),
                MultiSendPayment::cat(asset_id, bob, 400),
                MultiSendPayment::cat(asset_id, alice.puzzle_hash, 100),
            ],
            50,
        )
        .await?;
    vault_transact(&mut vault, coin_spends).await?;

    assert_eq!(vault.wallet.db.xch_balance().await?, 1450);
    assert_eq!(vault.wallet.db.cat_balance(asset_id).await?, 500);

    Ok(())
}

#[test(tokio::test)]
async fn test_vault_split_and_combine() -> anyhow::Result<()> {
    let mut alice = TestWallet::new(10_000).await?;
    let mut vault = mint(
        &mut alice,
        &config(VaultVersion::ForceSingletonRecreation),
        3000,
    )
    .await?;

    let coin_ids = vault
        .wallet
        .db
        .selectable_xch_coins()
        .await?
        .iter()
        .map(Coin::coin_id)
        .collect();

    let coin_spends = vault.wallet.split(coin_ids, 3, 0).await?;
    vault_transact(&mut vault, coin_spends).await?;

    let coins = vault.wallet.db.selectable_xch_coins().await?;
    assert_eq!(coins.len(), 3);
    assert!(
        coins
            .iter()
            .all(|coin| coin.puzzle_hash == vault.puzzle_hash)
    );
    assert_eq!(vault.wallet.db.xch_balance().await?, 3000);

    let coin_spends = vault
        .wallet
        .combine(coins.iter().map(Coin::coin_id).collect(), 100)
        .await?;
    vault_transact(&mut vault, coin_spends).await?;

    let coins = vault.wallet.db.selectable_xch_coins().await?;
    assert_eq!(coins.len(), 1);
    assert_eq!(vault.wallet.db.xch_balance().await?, 2900);

    Ok(())
}

/// The custody signs over the vault's coin id, so a second spend made before the first confirms
/// would spend the same coin, and is refused.
#[test(tokio::test)]
async fn test_vault_spend_pending() -> anyhow::Result<()> {
    let mut alice = TestWallet::new(10_000).await?;
    let vault = mint(
        &mut alice,
        &config(VaultVersion::ForceSingletonRecreation),
        3000,
    )
    .await?;

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
        .send_xch(vec![(alice.puzzle_hash, 500)], 0, vec![], None)
        .await;
    assert!(
        matches!(result, Err(WalletError::VaultSpendPending)),
        "{result:?}"
    );

    Ok(())
}

/// The vault singleton is its own asset, so it's never selected as XCH, and can't be spent as a
/// coin of the wallet.
#[test(tokio::test)]
async fn test_vault_singleton_not_selectable() -> anyhow::Result<()> {
    let mut alice = TestWallet::new(10_000).await?;
    let vault = mint(
        &mut alice,
        &config(VaultVersion::ForceSingletonRecreation),
        1000,
    )
    .await?;

    let launcher_id = vault.wallet.vault_launcher_id().expect("a vault wallet");
    let vault_coin = vault
        .wallet
        .db
        .vault_status(launcher_id)
        .await?
        .and_then(|status| status.coin)
        .expect("the vault coin is synced")
        .coin;

    let coins = vault.wallet.db.selectable_xch_coins().await?;
    assert!(
        coins
            .iter()
            .all(|coin| coin.coin_id() != vault_coin.coin_id())
    );
    assert_eq!(vault.wallet.db.xch_balance().await?, 1000);

    let result = vault
        .wallet
        .send_xch_with_coins(
            vec![(alice.puzzle_hash, 1)],
            0,
            vec![],
            None,
            vec![vault_coin.coin_id()],
        )
        .await;
    assert!(
        matches!(result, Err(WalletError::VaultSpendNotSupported)),
        "{result:?}"
    );

    // More than the balance can't be made up with the singleton's mojo.
    let result = vault
        .wallet
        .send_xch(vec![(alice.puzzle_hash, 1001)], 0, vec![], None)
        .await;
    assert!(result.is_err(), "{result:?}");

    Ok(())
}

/// A wallet without the custody key builds the spend, but can't sign it.
#[test(tokio::test)]
async fn test_vault_wrong_signer() -> anyhow::Result<()> {
    let mut alice = TestWallet::new(10_000).await?;
    let mint_result = alice
        .wallet
        .mint_vault(&config(VaultVersion::ForceSingletonRecreation), 0)
        .await?;
    alice.transact(mint_result.coin_spends).await?;
    alice.wait_for_coins().await;

    let mut vault = alice
        .vault_wallet(mint_result.launcher_id, recovery_key())
        .await?;
    wait_for_vault(&mut vault).await?;

    let coin_spends = vault.wallet.combine(vec![], 0).await?;
    let result = vault.sign(coin_spends).await;
    assert!(result.is_err(), "{result:?}");

    Ok(())
}

#[test(tokio::test)]
async fn test_vault_clear_signing() -> anyhow::Result<()> {
    let mut alice = TestWallet::new(10_000).await?;
    let config = config(VaultVersion::ForceSingletonRecreation);
    let vault = mint(&mut alice, &config, 3000).await?;
    let launcher_id = vault.wallet.vault_launcher_id().expect("a vault wallet");

    let coin_spends = vault
        .wallet
        .send_xch(vec![(alice.puzzle_hash, 1000)], 100, vec![], None)
        .await?;

    let summary = parse_vault_spend_bundle(&coin_spends, launcher_id)?.expect("the vault is spent");
    assert!(!summary.changes_custody());
    assert_eq!(summary.reserved_fee, 100);
    assert_eq!(summary.unaccounted_xch, 0);

    // Alice's own spends don't spend the vault.
    let coin_spends = alice
        .wallet
        .send_xch(vec![(alice.puzzle_hash, 1)], 0, vec![], None)
        .await?;
    assert!(parse_vault_spend_bundle(&coin_spends, launcher_id)?.is_none());

    // A spend that recreates the vault with another custody is flagged.
    let spendable = vault.wallet.spendable_vault().await?;
    let mut ctx = SpendContext::new();
    let delegated_spend = ctx.delegated_spend(
        Conditions::new()
            .create_coin(Bytes32::new([1; 32]), 1, Memos::None)
            .assert_my_amount(1),
    )?;
    let mips_spend = custody_mips_spend(
        &mut ctx,
        &config.custody,
        vault_hashes(&config)?.recovery_member_hash,
        delegated_spend,
        &[custody_key().public_key()],
    )?;
    spendable.vault.spend(&mut ctx, &mips_spend)?;

    let summary = parse_vault_spend_bundle(&ctx.take(), launcher_id)?.expect("the vault is spent");
    assert!(summary.changes_custody());
    assert_eq!(summary.child_custody_hash, Some(Bytes32::new([1; 32])));

    Ok(())
}

/// Coins sent to the vault while its wallet is closed are synced when it's opened again.
#[test(tokio::test)]
async fn test_vault_receives_while_closed() -> anyhow::Result<()> {
    let mut alice = TestWallet::new(10_000).await?;
    let mut vault = mint(
        &mut alice,
        &config(VaultVersion::ForceSingletonRecreation),
        0,
    )
    .await?;

    vault.stop_sync().await?;

    // The simulator confirms transactions at the height of its last block, which the vault wallet
    // may have already synced up to, so a new block is made first, as a real chain would.
    alice.sim.lock().await.create_block();

    let coin_spends = alice
        .wallet
        .send_xch(vec![(vault.puzzle_hash, 1000)], 0, vec![], None)
        .await?;
    alice.transact(coin_spends).await?;
    alice.wait_for_coins().await;

    vault.restart_sync().await?;

    for _ in 0..50 {
        if vault.wallet.db.xch_balance().await? == 1000 {
            return Ok(());
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    anyhow::bail!("the vault didn't sync the coin sent while it was closed");
}
