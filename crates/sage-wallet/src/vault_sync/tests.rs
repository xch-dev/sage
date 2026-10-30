use std::{slice, time::Duration};

use chia_wallet_sdk::{
    chia::puzzle_types::nft::NftMetadata,
    driver::{
        InnerPuzzleSpend, MipsSpend, P2ConditionsOrSingleton, P2ConditionsOrSingletonVersion,
        Restriction, RestrictionKind, Vault, VaultVersion,
    },
    prelude::*,
    test::{BlsPair, sign_transaction},
    types::{
        Mod,
        puzzles::{
            BlsMember, Force1of2RestrictedVariable, Force1of2RestrictedVariableSolution,
            ForceSingletonRecreationMod, PreventConditionOpcode, PreventMultipleCreateCoinsMod,
            Timelock,
        },
    },
};
use sage_database::{
    P2Puzzle, VaultConfig, VaultLayout, VaultMember, VaultRecoveryConfig, VaultSigners, VaultState,
    VaultStatus,
};
use test_log::test;
use tokio::time::sleep;

use crate::{
    SyncCommand, SyncEvent, TestWallet, VAULT_ADDRESS_GAP, VaultHashes, WalletNftMint, build_vault,
    build_vault_recovery, p2_singleton_puzzle_hash, recovery_restrictions, track_vault,
    vault_hashes,
};

const TIMELOCK: u64 = 60;

fn vault_config(custody: &BlsPair, recovery: &BlsPair, version: VaultVersion) -> VaultConfig {
    VaultConfig {
        custody: VaultSigners::new(vec![VaultMember::bls(custody.pk)], 1),
        recovery: VaultSigners::new(vec![VaultMember::bls(recovery.pk)], 1),
        recovery_timelock: TIMELOCK,
        version,
    }
}

/// The current coin of a vault, as the test spends it.
#[derive(Debug, Clone, Copy)]
struct TestVault {
    vault: Vault,
}

impl TestVault {
    fn launcher_id(&self) -> Bytes32 {
        self.vault.info.launcher_id
    }

    fn custody_hash(&self) -> Bytes32 {
        self.vault.info.custody_hash.into()
    }

    fn child(&self, custody_hash: Bytes32) -> Self {
        Self {
            vault: self
                .vault
                .child(custody_hash.into(), self.vault.coin.amount),
        }
    }
}

async fn push(
    test: &TestWallet,
    coin_spends: Vec<CoinSpend>,
    keys: &[SecretKey],
) -> anyhow::Result<()> {
    let signature = sign_transaction(&coin_spends, keys)?;
    let ack = test
        .peer
        .send_transaction(SpendBundle::new(coin_spends, signature))
        .await?;
    anyhow::ensure!(ack.status == 1, "transaction failed: {:?}", ack.error);
    Ok(())
}

/// Mints a vault from a coin that isn't the wallet's, with its memo in the launcher solution
/// unless `memo` is false.
async fn mint(test: &TestWallet, config: &VaultConfig, memo: bool) -> anyhow::Result<TestVault> {
    let funder = BlsPair::new(1000);
    let coin = test
        .sim
        .lock()
        .await
        .new_coin(StandardLayer::new(funder.pk).tree_hash().into(), 1);

    let mut ctx = SpendContext::new();
    let built = build_vault(&mut ctx, config)?;
    let memos = if memo { built.memo_list } else { NodePtr::NIL };

    let (conditions, vault) =
        Launcher::new(coin.coin_id(), 1).mint_vault(&mut ctx, built.hashes.custody_hash, memos)?;
    StandardLayer::new(funder.pk).spend(&mut ctx, coin, conditions)?;

    push(test, ctx.take(), &[funder.sk]).await?;

    Ok(TestVault { vault })
}

/// Tracks a vault in the running wallet, as importing it does.
async fn track(test: &TestWallet, launcher_id: Bytes32) -> anyhow::Result<()> {
    let mut tx = test.wallet.db.tx().await?;
    let puzzle_hashes = track_vault(&mut tx, launcher_id).await?;
    tx.commit().await?;

    test.sender
        .send(SyncCommand::SubscribeCoins {
            coin_ids: vec![launcher_id],
        })
        .await?;
    test.sender
        .send(SyncCommand::SubscribePuzzles { puzzle_hashes })
        .await?;

    Ok(())
}

fn bls_member(ctx: &mut SpendContext, pk: PublicKey) -> anyhow::Result<Spend> {
    Ok(Spend::new(ctx.curry(BlsMember::new(pk))?, NodePtr::NIL))
}

/// Spends a vault through its custody, which is the left side of the top level 1 of 2.
async fn custody_spend(
    test: &TestWallet,
    vault: &TestVault,
    hashes: VaultHashes,
    custody: &BlsPair,
    build: impl FnOnce(&mut SpendContext) -> anyhow::Result<Conditions>,
) -> anyhow::Result<()> {
    let mut ctx = SpendContext::new();
    let conditions = build(&mut ctx)?;

    let mut spend = MipsSpend::new(ctx.delegated_spend(conditions)?);
    spend.members.insert(
        hashes.custody_hash,
        InnerPuzzleSpend::m_of_n(
            0,
            vec![],
            1,
            vec![hashes.custody_member_hash, hashes.recovery_member_hash],
        ),
    );
    let member = bls_member(&mut ctx, custody.pk)?;
    spend.members.insert(
        hashes.custody_member_hash,
        InnerPuzzleSpend::new(0, vec![], member),
    );

    vault.vault.spend(&mut ctx, &spend)?;
    push(test, ctx.take(), slice::from_ref(&custody.sk)).await
}

/// Recreates the vault with a new config, revealing it in a memo.
async fn rekey(
    test: &TestWallet,
    vault: &TestVault,
    config: &VaultConfig,
    custody: &BlsPair,
    new_config: &VaultConfig,
) -> anyhow::Result<TestVault> {
    let new_custody_hash = vault_hashes(new_config)?.custody_hash.into();
    let amount = vault.vault.coin.amount;

    custody_spend(test, vault, vault_hashes(config)?, custody, |ctx| {
        let memo_list = build_vault(ctx, new_config)?.memo_list;
        Ok(Conditions::new().create_coin(new_custody_hash, amount, Memos::Some(memo_list)))
    })
    .await?;

    Ok(vault.child(new_custody_hash))
}

/// Starts recovery with the recovery key, moving the vault to the recovery state, which can move it
/// to `post_recovery` once the timelock expires.
async fn start_recovery(
    test: &TestWallet,
    vault: &TestVault,
    config: &VaultConfig,
    recovery: &BlsPair,
    post_recovery: &VaultConfig,
) -> anyhow::Result<TestVault> {
    let mut ctx = SpendContext::new();
    let hashes = vault_hashes(config)?;

    let recovery_config = VaultRecoveryConfig {
        custody: config.custody.clone(),
        recovery_timelock: config.recovery_timelock,
        version: config.version,
        recovery_custody_hash: vault_hashes(post_recovery)?.custody_hash.into(),
        amount: vault.vault.coin.amount,
    };
    let built = build_vault_recovery(&mut ctx, &recovery_config, post_recovery)?;
    let recovery_custody_hash: Bytes32 = built.hashes.custody_hash.into();

    // Force singleton recreation requires the vault to assert its amount, which it recreates.
    let conditions = Conditions::new()
        .create_coin(
            recovery_custody_hash,
            vault.vault.coin.amount,
            Memos::Some(built.memo_list),
        )
        .assert_my_amount(vault.vault.coin.amount);
    let mut spend = MipsSpend::new(ctx.delegated_spend(conditions)?);

    spend.members.insert(
        hashes.custody_hash,
        InnerPuzzleSpend::m_of_n(
            0,
            vec![],
            1,
            vec![hashes.custody_member_hash, hashes.recovery_member_hash],
        ),
    );
    let member = bls_member(&mut ctx, recovery.pk)?;
    spend.members.insert(
        hashes.recovery_member_hash,
        InnerPuzzleSpend::new(
            0,
            recovery_restrictions(
                hashes.custody_member_hash,
                config.recovery_timelock,
                config.version,
            ),
            member,
        ),
    );

    // The recovery path's delegated puzzle wrappers, in order.
    let force_1_of_2 = Force1of2RestrictedVariable::new(
        hashes.custody_member_hash.into(),
        0,
        vec![Timelock::new(config.recovery_timelock).curry_tree_hash()]
            .tree_hash()
            .into(),
        ().tree_hash().into(),
    );
    let puzzle = ctx.curry(force_1_of_2)?;
    // The wrapper rebuilds the timelocked right side from the finish member's puzzle hash.
    let finish_puzzle_hash = ctx.tree_hash(built.recovery_finish_puzzle.unwrap()).into();
    let solution = ctx.alloc(&Force1of2RestrictedVariableSolution::new(
        finish_puzzle_hash,
    ))?;
    spend
        .restrictions
        .insert(force_1_of_2.curry_tree_hash(), Spend::new(puzzle, solution));

    for opcode in [60, 62, 66, 67] {
        let wrapper = PreventConditionOpcode::new(opcode);
        let puzzle = ctx.curry(wrapper)?;
        spend
            .restrictions
            .insert(wrapper.curry_tree_hash(), Spend::new(puzzle, NodePtr::NIL));
    }

    let puzzle = ctx.alloc_mod::<PreventMultipleCreateCoinsMod>()?;
    spend.restrictions.insert(
        PreventMultipleCreateCoinsMod::mod_hash(),
        Spend::new(puzzle, NodePtr::NIL),
    );

    if config.version == VaultVersion::ForceSingletonRecreation {
        let puzzle = ctx.alloc_mod::<ForceSingletonRecreationMod>()?;
        spend.restrictions.insert(
            ForceSingletonRecreationMod::mod_hash(),
            Spend::new(puzzle, NodePtr::NIL),
        );
    }

    vault.vault.spend(&mut ctx, &spend)?;
    push(test, ctx.take(), slice::from_ref(&recovery.sk)).await?;

    Ok(vault.child(recovery_custody_hash))
}

/// The hashes and finish puzzle of a vault in the recovery state.
fn recovery_state(
    ctx: &mut SpendContext,
    config: &VaultConfig,
    post_recovery: &VaultConfig,
    amount: u64,
) -> anyhow::Result<(VaultHashes, NodePtr)> {
    let recovery_config = VaultRecoveryConfig {
        custody: config.custody.clone(),
        recovery_timelock: config.recovery_timelock,
        version: config.version,
        recovery_custody_hash: vault_hashes(post_recovery)?.custody_hash.into(),
        amount,
    };
    let built = build_vault_recovery(ctx, &recovery_config, post_recovery)?;
    Ok((built.hashes, built.recovery_finish_puzzle.unwrap()))
}

/// Cancels recovery with the original custody, recreating the vault with its original config.
async fn cancel_recovery(
    test: &TestWallet,
    vault: &TestVault,
    config: &VaultConfig,
    custody: &BlsPair,
    post_recovery: &VaultConfig,
) -> anyhow::Result<TestVault> {
    let mut ctx = SpendContext::new();
    let (hashes, _) = recovery_state(&mut ctx, config, post_recovery, vault.vault.coin.amount)?;
    let custody_hash = vault_hashes(config)?.custody_hash.into();
    let amount = vault.vault.coin.amount;

    custody_spend(test, vault, hashes, custody, |ctx| {
        let memo_list = build_vault(ctx, config)?.memo_list;
        Ok(Conditions::new().create_coin(custody_hash, amount, Memos::Some(memo_list)))
    })
    .await?;

    Ok(vault.child(custody_hash))
}

/// Finishes recovery once the timelock has expired, which anyone can do without a signature.
async fn finish_recovery(
    test: &TestWallet,
    vault: &TestVault,
    config: &VaultConfig,
    post_recovery: &VaultConfig,
) -> anyhow::Result<TestVault> {
    let mut ctx = SpendContext::new();
    let (hashes, finish_puzzle) =
        recovery_state(&mut ctx, config, post_recovery, vault.vault.coin.amount)?;

    let mut spend = MipsSpend::new(ctx.delegated_spend(Conditions::new())?);
    spend.members.insert(
        hashes.custody_hash,
        InnerPuzzleSpend::m_of_n(
            0,
            vec![],
            1,
            vec![hashes.custody_member_hash, hashes.recovery_member_hash],
        ),
    );

    let timelock = Timelock::new(config.recovery_timelock);
    spend.members.insert(
        hashes.recovery_member_hash,
        InnerPuzzleSpend::new(
            0,
            vec![Restriction {
                kind: RestrictionKind::MemberCondition,
                puzzle_hash: timelock.curry_tree_hash(),
            }],
            Spend::new(finish_puzzle, NodePtr::NIL),
        ),
    );
    let puzzle = ctx.curry(timelock)?;
    spend
        .restrictions
        .insert(timelock.curry_tree_hash(), Spend::new(puzzle, NodePtr::NIL));

    vault.vault.spend(&mut ctx, &spend)?;
    push(test, ctx.take(), &[]).await?;

    Ok(vault.child(vault_hashes(post_recovery)?.custody_hash.into()))
}

async fn wait_for_vault(
    test: &TestWallet,
    launcher_id: Bytes32,
    f: impl Fn(&VaultStatus) -> bool,
) -> anyhow::Result<VaultStatus> {
    let mut last = None;

    for _ in 0..100 {
        // The test database uses a shared cache, where a read transaction can deadlock with sync's
        // writes, so this takes the write lock up front instead.
        let mut tx = test.wallet.db.tx().await?;
        let status = tx.vault_status(launcher_id).await?;
        tx.commit().await?;

        if let Some(status) = &status
            && f(status)
        {
            return Ok(status.clone());
        }

        last = status;
        sleep(Duration::from_millis(100)).await;
    }

    anyhow::bail!("timed out waiting for the vault, which is {last:?}");
}

/// Waits until the vault's current coin is the expected one, with the expected state.
async fn wait_for_coin(
    test: &TestWallet,
    vault: &TestVault,
    state: impl Fn(VaultState) -> bool,
) -> anyhow::Result<VaultStatus> {
    let coin = vault.vault.coin;
    let custody_hash = vault.custody_hash();

    let status = wait_for_vault(test, vault.launcher_id(), |status| {
        status.coin.is_some_and(|row| {
            row.coin == coin && row.created_height.is_some() && row.spent_height.is_none()
        }) && state(status.state)
    })
    .await?;

    assert_eq!(status.coin.unwrap().custody_hash, Some(custody_hash));

    Ok(status)
}

async fn wait_for_xch_coin(test: &TestWallet, coin_id: Bytes32) -> anyhow::Result<()> {
    for _ in 0..100 {
        if test.wallet.db.xch_coin(coin_id).await?.is_some() {
            return Ok(());
        }
        sleep(Duration::from_millis(100)).await;
    }

    anyhow::bail!("timed out waiting for coin {coin_id}");
}

fn is_active(state: VaultState) -> bool {
    state == VaultState::Active
}

fn is_recovery_pending(state: VaultState) -> bool {
    matches!(state, VaultState::RecoveryPending { .. })
}

/// Follows a vault through every kind of spend, checking its custody hash, config and state.
async fn lifecycle(version: VaultVersion) -> anyhow::Result<()> {
    let mut test = TestWallet::new(0).await?;

    let custody = BlsPair::new(1);
    let recovery = BlsPair::new(2);
    let new_custody = BlsPair::new(3);
    let config = vault_config(&custody, &recovery, version);
    let new_config = vault_config(&new_custody, &recovery, version);

    let vault = mint(&test, &config, true).await?;
    track(&test, vault.launcher_id()).await?;

    // The eve coin's config is decoded from the launcher's memo.
    let status = wait_for_coin(&test, &vault, is_active).await?;
    assert_eq!(
        status.config.map(|row| row.layout),
        Some(VaultLayout::Custody(config.clone()))
    );

    // A plain custody spend keeps its config.
    let custody_hash = vault.custody_hash();
    let amount = vault.vault.coin.amount;
    custody_spend(&test, &vault, vault_hashes(&config)?, &custody, |_| {
        Ok(Conditions::new().create_coin(custody_hash, amount, Memos::None))
    })
    .await?;
    let vault = vault.child(custody_hash);
    let status = wait_for_coin(&test, &vault, is_active).await?;
    assert!(status.config.is_some());

    // Starting recovery reveals the recovery state, and the custody it moves to.
    let vault = start_recovery(&test, &vault, &config, &recovery, &new_config).await?;
    test.consume_until(|event| {
        matches!(
            event,
            SyncEvent::VaultStateChanged {
                state: VaultState::RecoveryPending { .. },
                ..
            }
        )
    })
    .await;
    let status = wait_for_coin(&test, &vault, is_recovery_pending).await?;
    let Some(VaultLayout::Recovery(recovery_config)) = status.config.map(|row| row.layout) else {
        panic!("expected the recovery state's config");
    };
    assert_eq!(
        recovery_config.recovery_custody_hash,
        Bytes32::from(vault_hashes(&new_config)?.custody_hash)
    );
    assert_eq!(recovery_config.recovery_timelock, TIMELOCK);

    // The original custody can cancel it.
    let vault = cancel_recovery(&test, &vault, &config, &custody, &new_config).await?;
    wait_for_coin(&test, &vault, is_active).await?;

    // Once the timelock of a new recovery has expired, anyone can finish it.
    let vault = start_recovery(&test, &vault, &config, &recovery, &new_config).await?;
    wait_for_coin(&test, &vault, is_recovery_pending).await?;

    test.sim.lock().await.pass_time(TIMELOCK + 1);

    let vault = finish_recovery(&test, &vault, &config, &new_config).await?;
    let status = wait_for_coin(&test, &vault, is_active).await?;
    assert_eq!(
        status.config.map(|row| row.layout),
        Some(VaultLayout::Custody(new_config.clone()))
    );

    // Rekeying moves it back to the original custody.
    let vault = rekey(&test, &vault, &new_config, &new_custody, &config).await?;
    let status = wait_for_coin(&test, &vault, is_active).await?;
    assert_eq!(
        status.config.map(|row| row.layout),
        Some(VaultLayout::Custody(config.clone()))
    );

    // Melting it leaves the melted coin as the latest one.
    custody_spend(&test, &vault, vault_hashes(&config)?, &custody, |_| {
        Ok(Conditions::new().with(MeltSingleton::default()))
    })
    .await?;
    let status = wait_for_vault(&test, vault.launcher_id(), |status| {
        status.state == VaultState::Melted
    })
    .await?;
    assert_eq!(status.coin.unwrap().coin, vault.vault.coin);

    // Vault coins are never part of the XCH balance.
    assert_eq!(test.wallet.db.xch_balance().await?, 0);

    Ok(())
}

#[test(tokio::test)]
async fn test_vault_lifecycle() -> anyhow::Result<()> {
    Box::pin(lifecycle(VaultVersion::ForceSingletonRecreation)).await
}

#[test(tokio::test)]
async fn test_legacy_vault_lifecycle() -> anyhow::Result<()> {
    Box::pin(lifecycle(VaultVersion::Legacy)).await
}

#[test(tokio::test)]
async fn test_vault_without_memos() -> anyhow::Result<()> {
    let test = TestWallet::new(0).await?;

    let custody = BlsPair::new(1);
    let config = vault_config(
        &custody,
        &BlsPair::new(2),
        VaultVersion::ForceSingletonRecreation,
    );

    let vault = mint(&test, &config, false).await?;
    track(&test, vault.launcher_id()).await?;

    // Without a memo, the eve coin's custody is unknown until it's spent.
    let status = wait_for_vault(&test, vault.launcher_id(), |status| {
        status.coin.is_some_and(|row| row.coin == vault.vault.coin)
    })
    .await?;
    assert_eq!(status.state, VaultState::Active);
    assert_eq!(status.coin.unwrap().custody_hash, None);
    assert_eq!(status.config, None);

    // Its addresses are tracked regardless, so payments to it are found.
    let address = p2_singleton_puzzle_hash(vault.launcher_id(), 0);
    let funder = BlsPair::new(1000);
    let funding = test
        .sim
        .lock()
        .await
        .new_coin(StandardLayer::new(funder.pk).tree_hash().into(), 250);
    let mut ctx = SpendContext::new();
    StandardLayer::new(funder.pk).spend(
        &mut ctx,
        funding,
        Conditions::new().create_coin(address, 250, Memos::None),
    )?;
    push(&test, ctx.take(), &[funder.sk]).await?;

    let paid = Coin::new(funding.coin_id(), address, 250);
    wait_for_xch_coin(&test, paid.coin_id()).await?;
    assert_eq!(test.wallet.db.xch_balance().await?, 250);

    // Spending it reveals the eve coin's custody, but not a config for the new coin.
    let custody_hash = vault.custody_hash();
    let amount = vault.vault.coin.amount;
    custody_spend(&test, &vault, vault_hashes(&config)?, &custody, |_| {
        Ok(Conditions::new().create_coin(custody_hash, amount, Memos::None))
    })
    .await?;
    let vault = vault.child(custody_hash);
    let status = wait_for_coin(&test, &vault, is_active).await?;
    assert_eq!(status.config, None);

    let coins = test.wallet.db.vault_coins(vault.launcher_id()).await?;
    assert_eq!(coins.len(), 2);
    assert_eq!(coins[0].custody_hash, Some(custody_hash));

    Ok(())
}

/// Polls until a condition on the wallet's database holds.
async fn wait_until<F, Fut>(f: F) -> anyhow::Result<()>
where
    F: Fn() -> Fut,
    Fut: Future<Output = anyhow::Result<bool>>,
{
    for _ in 0..100 {
        if f().await? {
            return Ok(());
        }
        sleep(Duration::from_millis(100)).await;
    }

    anyhow::bail!("timed out");
}

#[test(tokio::test)]
async fn test_vault_addresses() -> anyhow::Result<()> {
    let test = TestWallet::new(0).await?;
    let mut sender = test.next(1500).await?;

    let config = vault_config(
        &BlsPair::new(1),
        &BlsPair::new(2),
        VaultVersion::ForceSingletonRecreation,
    );
    let vault = mint(&test, &config, true).await?;
    let launcher_id = vault.launcher_id();
    track(&test, launcher_id).await?;
    wait_for_coin(&test, &vault, is_active).await?;

    let db = &test.wallet.db;

    assert_eq!(
        db.p2_singleton_puzzle_hashes(launcher_id).await?.len(),
        VAULT_ADDRESS_GAP as usize
    );

    // XCH sent to the first address and to a later one.
    let first = p2_singleton_puzzle_hash(launcher_id, 0);
    let later = p2_singleton_puzzle_hash(launcher_id, 7);
    let coin_spends = sender
        .wallet
        .send_xch(vec![(first, 100), (later, 200)], 0, vec![], None)
        .await?;
    sender.transact(coin_spends).await?;
    sender.wait_for_coins().await;

    // A CAT, which is hinted to the vault's address.
    let (coin_spends, asset_id) = sender.wallet.issue_cat(500, 0, None).await?;
    sender.transact(coin_spends).await?;
    sender.wait_for_coins().await;
    let coin_spends = sender
        .wallet
        .send_cat(asset_id, vec![(later, 500)], 0, true, vec![], None)
        .await?;
    sender.transact(coin_spends).await?;
    sender.wait_for_coins().await;

    // An NFT, minted straight to the vault's address.
    let (coin_spends, did) = sender.wallet.create_did(0).await?;
    sender.transact(coin_spends).await?;
    sender.wait_for_coins().await;
    let (coin_spends, nfts) = sender
        .wallet
        .bulk_mint_nfts(
            0,
            did.info.launcher_id,
            vec![WalletNftMint {
                metadata: NftMetadata::default(),
                p2_puzzle_hash: Some(first),
                royalty_puzzle_hash: Some(Bytes32::default()),
                royalty_basis_points: 0,
            }],
        )
        .await?;
    sender.transact(coin_spends).await?;
    sender.wait_for_coins().await;

    let nft_id = nfts[0].info.launcher_id;

    wait_until(|| async {
        Ok(db.xch_balance().await? == 300
            && db.cat_balance(asset_id).await? == 500
            && db.spendable_nft(nft_id).await?.is_some())
    })
    .await?;

    let cats = db.selectable_cat_coins(asset_id).await?;
    assert_eq!(cats.len(), 1);
    assert_eq!(cats[0].info.p2_puzzle_hash, later);

    let nft = db.spendable_nft(nft_id).await?.unwrap();
    assert_eq!(nft.info.p2_puzzle_hash, first);

    assert!(matches!(
        db.p2_puzzle(later).await?,
        P2Puzzle::P2Singleton(p2) if p2.launcher_id == launcher_id && p2.nonce == 7
    ));

    // Paying the last address extends the gap, so payments past it are found too.
    let last = p2_singleton_puzzle_hash(launcher_id, (VAULT_ADDRESS_GAP - 1) as usize);
    let coin_spends = sender
        .wallet
        .send_xch(vec![(last, 10)], 0, vec![], None)
        .await?;
    sender.transact(coin_spends).await?;
    sender.wait_for_coins().await;

    wait_until(|| async {
        Ok(db.p2_singleton_puzzle_hashes(launcher_id).await?.len()
            == 2 * VAULT_ADDRESS_GAP as usize)
    })
    .await?;

    let beyond = p2_singleton_puzzle_hash(launcher_id, (VAULT_ADDRESS_GAP + 50) as usize);
    let coin_spends = sender
        .wallet
        .send_xch(vec![(beyond, 20)], 0, vec![], None)
        .await?;
    sender.transact(coin_spends).await?;
    sender.wait_for_coins().await;

    wait_until(|| async { Ok(db.xch_balance().await? == 330) }).await?;
    assert!(db.is_custody_p2_puzzle_hash(beyond).await?);

    Ok(())
}

#[test(tokio::test)]
async fn test_vault_offer_coins() -> anyhow::Result<()> {
    let test = TestWallet::new(0).await?;

    let config = vault_config(
        &BlsPair::new(1),
        &BlsPair::new(2),
        VaultVersion::ForceSingletonRecreation,
    );
    let vault = mint(&test, &config, true).await?;
    let launcher_id = vault.launcher_id();
    track(&test, launcher_id).await?;
    wait_for_coin(&test, &vault, is_active).await?;

    let address = p2_singleton_puzzle_hash(launcher_id, 0);

    let funder = BlsPair::new(1000);
    let funding = test
        .sim
        .lock()
        .await
        .new_coin(StandardLayer::new(funder.pk).tree_hash().into(), 600);

    let mut ctx = SpendContext::new();

    // An offer's pre-split coin, whose memos reveal the vault's address and the fixed conditions.
    let fixed = Conditions::new().reserve_fee(400);
    let fixed_spend = ctx.delegated_spend(fixed.clone())?;
    let quoted_hash = ctx.tree_hash(fixed_spend.puzzle).into();
    let locked = P2ConditionsOrSingleton::from_quoted_conditions_hash(
        launcher_id,
        0,
        quoted_hash,
        P2ConditionsOrSingletonVersion::FixedPuzzleMember,
    );
    let locked_hash: Bytes32 = locked.tree_hash().into();
    let memos = ctx.memos(&(address, (fixed, ())))?;

    // A coin with the same memos which isn't locked to the vault is ignored.
    let unrelated = Bytes32::new([9; 32]);
    let unrelated_memos = ctx.memos(&(address, (Conditions::new().reserve_fee(1), ())))?;

    StandardLayer::new(funder.pk).spend(
        &mut ctx,
        funding,
        Conditions::new()
            .create_coin(locked_hash, 400, memos)
            .create_coin(unrelated, 200, unrelated_memos),
    )?;
    push(&test, ctx.take(), &[funder.sk]).await?;

    let locked_coin = Coin::new(funding.coin_id(), locked_hash, 400);
    wait_for_xch_coin(&test, locked_coin.coin_id()).await?;

    assert_eq!(test.wallet.db.xch_balance().await?, 400);
    assert!(matches!(
        test.wallet.db.p2_puzzle(locked_hash).await?,
        P2Puzzle::P2ConditionsOrSingleton(p2)
            if p2.p2 == locked && p2.version == Some(P2ConditionsOrSingletonVersion::FixedPuzzleMember)
    ));
    assert!(
        test.wallet
            .db
            .p2_conditions_or_singleton_conditions(locked_hash)
            .await?
            .is_some()
    );

    // The coin shows up as the vault address in the transaction history.
    assert_eq!(
        test.history_custody(locked_coin.coin_id()).await?,
        Some(address)
    );

    Ok(())
}

#[test(tokio::test)]
async fn test_vault_resumes_after_restart() -> anyhow::Result<()> {
    let mut test = TestWallet::new(0).await?;

    let custody = BlsPair::new(1);
    let config = vault_config(
        &custody,
        &BlsPair::new(2),
        VaultVersion::ForceSingletonRecreation,
    );

    let vault = mint(&test, &config, true).await?;
    track(&test, vault.launcher_id()).await?;
    wait_for_coin(&test, &vault, is_active).await?;

    // Once the launcher has been synced, the vault is followed from its stored lineage.
    assert!(
        test.wallet
            .db
            .unsynced_vault_launcher_ids()
            .await?
            .is_empty()
    );

    // The vault is spent while the wallet is closed.
    test.stop_sync().await?;

    let custody_hash = vault.custody_hash();
    let amount = vault.vault.coin.amount;
    custody_spend(&test, &vault, vault_hashes(&config)?, &custody, |_| {
        Ok(Conditions::new().create_coin(custody_hash, amount, Memos::None))
    })
    .await?;
    let vault = vault.child(custody_hash);

    test.restart_sync().await?;
    wait_for_coin(&test, &vault, is_active).await?;

    assert_eq!(
        test.wallet.db.vault_coins(vault.launcher_id()).await?.len(),
        2
    );

    Ok(())
}
