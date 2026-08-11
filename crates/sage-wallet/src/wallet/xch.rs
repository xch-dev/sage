use std::time::{SystemTime, UNIX_EPOCH};

use chia_wallet_sdk::{
    driver::{Clawback as ClawbackV1, ClawbackV2},
    prelude::*,
};
use sage_database::{CoinKind, Database, P2Puzzle};

use crate::{
    WalletError,
    wallet::memos::{Hint, calculate_memos},
};

use super::Wallet;

async fn ensure_clawback_unspent(db: &Database, coin_id: Bytes32) -> Result<(), WalletError> {
    let coin_id_hex = hex::encode(coin_id);
    let rows = db.coins_by_ids(&[coin_id_hex]).await?;
    let Some(row) = rows.first() else {
        return Err(WalletError::MissingCoin(coin_id));
    };
    if row.spent_height.is_some() || row.mempool_item_hash.is_some() {
        return Err(WalletError::ClawbackAlreadySpentOrPending(coin_id));
    }
    Ok(())
}

impl Wallet {
    /// Sends the given amount of XCH to the given puzzle hash, minus the fee.
    pub async fn send_xch(
        &self,
        amounts: Vec<(Bytes32, u64)>,
        fee: u64,
        memos: Vec<Bytes>,
        clawback: Option<u64>,
    ) -> Result<Vec<CoinSpend>, WalletError> {
        let sender_puzzle_hash = self.change_p2_puzzle_hash().await?;

        let mut ctx = SpendContext::new();
        let mut actions = vec![Action::fee(fee)];

        for (puzzle_hash, amount) in amounts {
            let clawback = clawback.map(|seconds| {
                ClawbackV2::new(sender_puzzle_hash, puzzle_hash, seconds, amount, false)
            });

            let memos = calculate_memos(
                &mut ctx,
                if let Some(clawback) = clawback {
                    Hint::Clawback(clawback)
                } else {
                    Hint::None
                },
                memos.clone(),
            )?;

            let p2_puzzle_hash = if let Some(clawback) = clawback {
                clawback.tree_hash().into()
            } else {
                puzzle_hash
            };

            actions.push(Action::send(Id::Xch, p2_puzzle_hash, amount, memos));
        }

        self.spend(&mut ctx, vec![], &actions).await?;

        Ok(ctx.take())
    }

    pub async fn finalize_clawback(
        &self,
        coin_ids: Vec<Bytes32>,
        fee: u64,
    ) -> Result<Vec<CoinSpend>, WalletError> {
        let mut ctx = SpendContext::new();

        for &coin_id in &coin_ids {
            let Some(coin_kind) = self.db.coin_kind(coin_id).await? else {
                return Err(WalletError::MissingCoin(coin_id));
            };

            match coin_kind {
                CoinKind::Xch => {
                    let Some(coin) = self.db.xch_coin(coin_id).await? else {
                        return Err(WalletError::MissingXchCoin(coin_id));
                    };
                    ensure_clawback_unspent(&self.db, coin_id).await?;

                    let P2Puzzle::Clawback(clawback) = self.db.p2_puzzle(coin.puzzle_hash).await?
                    else {
                        return Err(WalletError::MissingClawbackInfo(coin_id));
                    };
                    if clawback.version != 2 {
                        return Err(WalletError::ClawbackVersionMismatch {
                            coin_id,
                            expected: 2,
                            actual: clawback.version,
                        });
                    }
                    let clawback = ClawbackV2::new(
                        clawback.sender_puzzle_hash,
                        clawback.receiver_puzzle_hash,
                        clawback.seconds,
                        coin.amount,
                        false,
                    );

                    clawback.push_through_coin_spend(&mut ctx, coin)?;
                }
                CoinKind::Cat => {
                    let Some(cat) = self.db.cat_coin(coin_id).await? else {
                        return Err(WalletError::MissingCatCoin(coin_id));
                    };
                    ensure_clawback_unspent(&self.db, coin_id).await?;

                    let P2Puzzle::Clawback(clawback) =
                        self.db.p2_puzzle(cat.info.p2_puzzle_hash).await?
                    else {
                        return Err(WalletError::MissingClawbackInfo(coin_id));
                    };
                    if clawback.version != 2 {
                        return Err(WalletError::ClawbackVersionMismatch {
                            coin_id,
                            expected: 2,
                            actual: clawback.version,
                        });
                    }

                    let clawback = ClawbackV2::new(
                        clawback.sender_puzzle_hash,
                        clawback.receiver_puzzle_hash,
                        clawback.seconds,
                        cat.coin.amount,
                        true,
                    );

                    let spend = clawback.push_through_spend(&mut ctx)?;
                    Cat::spend_all(&mut ctx, &[CatSpend::new(cat, spend)])?;
                }
                _ => {
                    return Err(WalletError::UnsupportedClawbackCoinKind(coin_kind));
                }
            }
        }

        if fee > 0 {
            let actions = [Action::fee(fee)];

            let mut spends = self.prepare_spends(&mut ctx, vec![], &actions).await?;

            for &coin_id in &coin_ids {
                spends
                    .conditions
                    .required
                    .push(AssertConcurrentSpend::new(coin_id));
            }

            let deltas = spends.apply(&mut ctx, &actions)?;
            self.complete_spends(&mut ctx, &deltas, spends).await?;
        }

        Ok(ctx.take())
    }

    pub async fn claim_clawback(
        &self,
        coin_ids: Vec<Bytes32>,
        fee: u64,
    ) -> Result<Vec<CoinSpend>, WalletError> {
        let mut ctx = SpendContext::new();
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

        for &coin_id in &coin_ids {
            let Some(coin_kind) = self.db.coin_kind(coin_id).await? else {
                return Err(WalletError::MissingCoin(coin_id));
            };

            match coin_kind {
                CoinKind::Xch => {
                    let Some(coin) = self.db.xch_coin(coin_id).await? else {
                        return Err(WalletError::MissingXchCoin(coin_id));
                    };

                    let P2Puzzle::Clawback(clawback) = self.db.p2_puzzle(coin.puzzle_hash).await?
                    else {
                        return Err(WalletError::MissingClawbackInfo(coin_id));
                    };
                    if clawback.version != 1 {
                        return Err(WalletError::ClawbackVersionMismatch {
                            coin_id,
                            expected: 1,
                            actual: clawback.version,
                        });
                    }

                    let coin_id_hex = hex::encode(coin_id);
                    let rows = self.db.coins_by_ids(&[coin_id_hex]).await?;
                    let Some(row) = rows.first() else {
                        return Err(WalletError::MissingCoin(coin_id));
                    };
                    if row.spent_height.is_some() || row.mempool_item_hash.is_some() {
                        return Err(WalletError::ClawbackAlreadySpentOrPending(coin_id));
                    }
                    // Preflight relative lock when created_timestamp is known; otherwise defer to chain.
                    if let Some(created_timestamp) = row.created_timestamp {
                        if now < created_timestamp.saturating_add(clawback.seconds) {
                            return Err(WalletError::ClawbackNotYetClaimable(coin_id));
                        }
                    }

                    let Some(receiver_key) = self
                        .db
                        .public_key(clawback.receiver_puzzle_hash)
                        .await?
                    else {
                        return Err(WalletError::UnknownPublicKey);
                    };

                    let v1 = ClawbackV1::new(
                        clawback.seconds,
                        clawback.sender_puzzle_hash,
                        clawback.receiver_puzzle_hash,
                    );
                    let receiver_p2 = StandardLayer::new(receiver_key);
                    v1.claim_coin_spend(&mut ctx, coin, &receiver_p2, Conditions::new())?;
                }
                _ => {
                    return Err(WalletError::UnsupportedClawbackCoinKind(coin_kind));
                }
            }
        }

        if fee > 0 {
            let actions = [Action::fee(fee)];

            let mut spends = self.prepare_spends(&mut ctx, vec![], &actions).await?;

            for &coin_id in &coin_ids {
                spends
                    .conditions
                    .required
                    .push(AssertConcurrentSpend::new(coin_id));
            }

            let deltas = spends.apply(&mut ctx, &actions)?;
            self.complete_spends(&mut ctx, &deltas, spends).await?;
        }

        Ok(ctx.take())
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use chia_wallet_sdk::{
        chia::{
            bls::{DerivableKey, master_to_wallet_unhardened_intermediate},
            puzzle_types::DeriveSynthetic,
        },
        driver::{Clawback as ClawbackV1, SpendContext, StandardLayer},
        prelude::*,
    };
    use sage_database::{AssetFilter, CoinFilterMode, CoinSortMode};
    use test_log::test;
    use tokio::time::sleep;

    use crate::{TestWallet, WalletError};

    // Test-only: Sage never creates V1. Emits REMARK metadata + receiver discovery hint.
    // Returns (coin_id, info, inclusion_block_timestamp).
    async fn create_v1_clawback(
        sender: &TestWallet,
        receiver_puzzle_hash: Bytes32,
        amount: u64,
        timelock: u64,
    ) -> anyhow::Result<(Bytes32, ClawbackV1, u64)> {
        let coins = sender.wallet.db.selectable_xch_coins().await?;
        let coin = coins
            .into_iter()
            .find(|c| c.amount >= amount)
            .expect("sender needs a selectable coin");

        let intermediate = master_to_wallet_unhardened_intermediate(&sender.master_sk);
        let synthetic = intermediate.derive_unhardened(0).derive_synthetic();
        let sender_p2 = StandardLayer::new(synthetic.public_key());

        let clawback = ClawbackV1::new(timelock, sender.puzzle_hash, receiver_puzzle_hash);
        let clawback_ph: Bytes32 = clawback.tree_hash().into();

        let mut ctx = SpendContext::new();
        let hint = ctx.hint(receiver_puzzle_hash)?;
        let mut conditions = Conditions::new().create_coin(clawback_ph, amount, hint);
        conditions = conditions.with(clawback.get_remark_condition(&mut ctx)?);

        if coin.amount > amount {
            let change_hint = ctx.hint(sender.puzzle_hash)?;
            conditions = conditions.create_coin(
                sender.puzzle_hash,
                coin.amount - amount,
                change_hint,
            );
        }

        sender_p2.spend(&mut ctx, coin, conditions)?;
        let coin_spends = ctx.take();
        sender.transact(coin_spends).await?;
        // Inclusion block with wall-clock time (BlockTimeQueue is off in testing mode).
        let inclusion_timestamp = sender.new_block_with_current_time().await?;

        let parent_id = coin.coin_id();
        let clawback_coin_id = Coin::new(parent_id, clawback_ph, amount).coin_id();
        Ok((clawback_coin_id, clawback, inclusion_timestamp))
    }

    async fn wait_for_clawback_coin(
        wallet: &TestWallet,
        coin_id: Bytes32,
        filter: CoinFilterMode,
    ) -> anyhow::Result<()> {
        for _ in 0..50 {
            let (rows, _) = wallet
                .wallet
                .db
                .coin_records(
                    AssetFilter::Id(Bytes32::default()),
                    100,
                    0,
                    CoinSortMode::CreatedHeight,
                    false,
                    filter,
                )
                .await?;
            if rows.iter().any(|r| r.coin.coin_id() == coin_id) {
                return Ok(());
            }
            sleep(Duration::from_millis(100)).await;
        }
        anyhow::bail!("timed out waiting for clawback coin {}", hex::encode(coin_id));
    }

    /// Seed `blocks.timestamp` for the coin's created height.
    /// Test mode disables BlockTimeQueue, so relative-lock preflight never sees a timestamp otherwise.
    async fn seed_created_timestamp(
        wallet: &TestWallet,
        coin_id: Bytes32,
        timestamp: u64,
    ) -> anyhow::Result<()> {
        for _ in 0..50 {
            let rows = wallet
                .wallet
                .db
                .coins_by_ids(&[hex::encode(coin_id)])
                .await?;
            if let Some(row) = rows.first() {
                if let Some(height) = row.created_height {
                    let header_hash = wallet
                        .wallet
                        .db
                        .latest_peak()
                        .await?
                        .map(|(peak_height, hash)| {
                            if peak_height == height {
                                hash
                            } else {
                                Bytes32::default()
                            }
                        })
                        .unwrap_or_default();
                    wallet
                        .wallet
                        .db
                        .insert_block(height, header_hash, Some(timestamp.try_into()?), false)
                        .await?;
                    let rows = wallet
                        .wallet
                        .db
                        .coins_by_ids(&[hex::encode(coin_id)])
                        .await?;
                    if rows.first().and_then(|r| r.created_timestamp) == Some(timestamp) {
                        return Ok(());
                    }
                }
            }
            sleep(Duration::from_millis(100)).await;
        }
        anyhow::bail!(
            "timed out seeding created_timestamp on {}",
            hex::encode(coin_id)
        );
    }

    #[test(tokio::test)]
    async fn test_send_xch() -> anyhow::Result<()> {
        let mut test = TestWallet::new(1000).await?;

        let coin_spends = test
            .wallet
            .send_xch(vec![(test.puzzle_hash, 1000)], 0, vec![], None)
            .await?;

        assert_eq!(coin_spends.len(), 1);

        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        assert_eq!(test.wallet.db.xch_balance().await?, 1000);
        assert_eq!(test.wallet.db.selectable_xch_coins().await?.len(), 1);

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_send_xch_change() -> anyhow::Result<()> {
        let mut test = TestWallet::new(1000).await?;

        let coin_spends = test
            .wallet
            .send_xch(vec![(test.puzzle_hash, 250)], 250, vec![], None)
            .await?;

        assert_eq!(coin_spends.len(), 1);

        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        assert_eq!(test.wallet.db.xch_balance().await?, 750);
        assert_eq!(test.wallet.db.selectable_xch_coins().await?.len(), 2);

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_send_xch_hardened() -> anyhow::Result<()> {
        let mut test = TestWallet::new(1000).await?;

        let coin_spends = test
            .wallet
            .send_xch(vec![(test.hardened_puzzle_hash, 1000)], 0, vec![], None)
            .await?;

        assert_eq!(coin_spends.len(), 1);

        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        assert_eq!(test.wallet.db.xch_balance().await?, 1000);
        assert_eq!(test.wallet.db.selectable_xch_coins().await?.len(), 1);

        let coin_spends = test
            .wallet
            .send_xch(vec![(test.puzzle_hash, 1000)], 0, vec![], None)
            .await?;

        assert_eq!(coin_spends.len(), 1);

        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        assert_eq!(test.wallet.db.xch_balance().await?, 1000);
        assert_eq!(test.wallet.db.selectable_xch_coins().await?.len(), 1);

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_send_xch_with_clawback_self() -> anyhow::Result<()> {
        let mut test = TestWallet::new(1000).await?;

        let timestamp = test.new_block_with_current_time().await?;

        let coin_spends = test
            .wallet
            .send_xch(
                vec![(test.puzzle_hash, 1000)],
                0,
                vec![],
                Some(timestamp + 5),
            )
            .await?;

        assert_eq!(coin_spends.len(), 1);

        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        assert_eq!(test.wallet.db.selectable_xch_balance().await?, 0);
        assert_eq!(test.wallet.db.selectable_xch_coins().await?.len(), 0);

        sleep(Duration::from_secs(6)).await;
        test.new_block_with_current_time().await?;

        assert_eq!(test.wallet.db.selectable_xch_balance().await?, 1000);
        assert_eq!(test.wallet.db.selectable_xch_coins().await?.len(), 1);

        let coin_spends = test
            .wallet
            .send_xch(vec![(test.puzzle_hash, 1000)], 0, vec![], None)
            .await?;

        assert_eq!(coin_spends.len(), 1);

        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        assert_eq!(test.wallet.db.xch_balance().await?, 1000);
        assert_eq!(test.wallet.db.selectable_xch_coins().await?.len(), 1);

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_send_xch_with_clawback_external() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(1000).await?;
        let mut bob = alice.next(0).await?;

        let timestamp = alice.new_block_with_current_time().await?;

        let coin_spends = alice
            .wallet
            .send_xch(
                vec![(bob.puzzle_hash, 1000)],
                0,
                vec![],
                Some(timestamp + 5),
            )
            .await?;

        assert_eq!(coin_spends.len(), 1);

        alice.transact(coin_spends).await?;

        alice.wait_for_coins().await;

        assert_eq!(alice.wallet.db.selectable_xch_balance().await?, 0);
        assert_eq!(alice.wallet.db.selectable_xch_coins().await?.len(), 0);

        bob.wait_for_puzzles().await;

        assert_eq!(bob.wallet.db.selectable_xch_balance().await?, 0);
        assert_eq!(bob.wallet.db.selectable_xch_coins().await?.len(), 0);

        sleep(Duration::from_secs(6)).await;
        bob.new_block_with_current_time().await?;

        assert_eq!(alice.wallet.db.selectable_xch_balance().await?, 0);
        assert_eq!(alice.wallet.db.selectable_xch_coins().await?.len(), 0);

        assert_eq!(bob.wallet.db.selectable_xch_balance().await?, 1000);
        assert_eq!(bob.wallet.db.selectable_xch_coins().await?.len(), 1);

        let coin_spends = bob
            .wallet
            .send_xch(vec![(alice.puzzle_hash, 1000)], 0, vec![], None)
            .await?;

        assert_eq!(coin_spends.len(), 1);

        bob.transact(coin_spends).await?;
        bob.wait_for_coins().await;
        alice.wait_for_coins().await;

        assert_eq!(alice.wallet.db.selectable_xch_balance().await?, 1000);
        assert_eq!(alice.wallet.db.selectable_xch_coins().await?.len(), 1);

        assert_eq!(bob.wallet.db.selectable_xch_balance().await?, 0);
        assert_eq!(bob.wallet.db.selectable_xch_coins().await?.len(), 0);

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_clawback_v1_sender_clawback() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(1000).await?;
        let mut bob = alice.next(0).await?;

        alice.new_block_with_current_time().await?;

        // Long relative lock (fits in i64); sender claws back before claim is possible.
        let (clawback_id, _, _) =
            create_v1_clawback(&alice, bob.puzzle_hash, 1000, 86_400 * 365 * 10).await?;

        alice.wait_for_coins().await;
        bob.wait_for_puzzles().await;

        wait_for_clawback_coin(&alice, clawback_id, CoinFilterMode::Clawback).await?;

        assert_eq!(alice.wallet.db.selectable_xch_balance().await?, 0);
        assert_eq!(bob.wallet.db.selectable_xch_balance().await?, 0);

        let coin_spends = alice.wallet.combine(vec![clawback_id], 0).await?;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        assert_eq!(alice.wallet.db.selectable_xch_balance().await?, 1000);
        assert_eq!(bob.wallet.db.selectable_xch_balance().await?, 0);

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_clawback_v1_sender_clawback_after_lock() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(1000).await?;
        let mut bob = alice.next(0).await?;

        alice.new_block_with_current_time().await?;

        // V1 sender may claw back even after the relative lock has elapsed (until claimed).
        let timelock = 1;
        let (clawback_id, _, _) =
            create_v1_clawback(&alice, bob.puzzle_hash, 1000, timelock).await?;

        alice.wait_for_coins().await;
        bob.wait_for_puzzles().await;
        wait_for_clawback_coin(&alice, clawback_id, CoinFilterMode::Clawback).await?;

        sleep(Duration::from_secs(timelock + 1)).await;
        alice.new_block_with_current_time().await?;

        assert_eq!(alice.wallet.db.selectable_xch_balance().await?, 0);
        assert_eq!(bob.wallet.db.selectable_xch_balance().await?, 0);

        let coin_spends = alice.wallet.combine(vec![clawback_id], 0).await?;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        assert_eq!(alice.wallet.db.selectable_xch_balance().await?, 1000);
        assert_eq!(bob.wallet.db.selectable_xch_balance().await?, 0);

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_clawback_v1_receiver_claim() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(1000).await?;
        let mut bob = alice.next(0).await?;

        alice.new_block_with_current_time().await?;

        let timelock = 1;
        let (clawback_id, _, inclusion_ts) =
            create_v1_clawback(&alice, bob.puzzle_hash, 1000, timelock).await?;

        alice.wait_for_coins().await;
        bob.wait_for_puzzles().await;
        seed_created_timestamp(&bob, clawback_id, inclusion_ts).await?;

        assert_eq!(bob.wallet.db.selectable_xch_balance().await?, 0);

        sleep(Duration::from_secs(timelock + 1)).await;
        bob.new_block_with_current_time().await?;

        let coin_spends = bob.wallet.claim_clawback(vec![clawback_id], 0).await?;
        assert!(!coin_spends.is_empty());
        bob.transact(coin_spends).await?;
        bob.wait_for_coins().await;

        assert_eq!(bob.wallet.db.selectable_xch_balance().await?, 1000);
        assert_eq!(alice.wallet.db.selectable_xch_balance().await?, 0);

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_clawback_v1_claim_before_lock() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(1000).await?;
        let mut bob = alice.next(0).await?;

        alice.new_block_with_current_time().await?;

        // Long enough that wall-clock now cannot satisfy preflight.
        let timelock = 3600;
        let (clawback_id, _, inclusion_ts) =
            create_v1_clawback(&alice, bob.puzzle_hash, 1000, timelock).await?;

        alice.wait_for_coins().await;
        bob.wait_for_puzzles().await;
        // Without this, testing mode leaves created_timestamp NULL and preflight is skipped.
        seed_created_timestamp(&bob, clawback_id, inclusion_ts).await?;

        let err = bob
            .wallet
            .claim_clawback(vec![clawback_id], 0)
            .await
            .expect_err("claim before relative lock should fail");
        assert!(
            matches!(err, WalletError::ClawbackNotYetClaimable(_)),
            "unexpected error: {err:?}"
        );
        assert_eq!(bob.wallet.db.selectable_xch_balance().await?, 0);

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_clawback_v1_finalize_rejects_v1() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(1000).await?;
        let bob = alice.next(0).await?;

        alice.new_block_with_current_time().await?;

        let (clawback_id, _, _) =
            create_v1_clawback(&alice, bob.puzzle_hash, 1000, 3600).await?;

        alice.wait_for_coins().await;
        wait_for_clawback_coin(&alice, clawback_id, CoinFilterMode::Clawback).await?;

        let err = alice
            .wallet
            .finalize_clawback(vec![clawback_id], 0)
            .await
            .expect_err("finalize is V2-only");
        assert!(matches!(
            err,
            WalletError::ClawbackVersionMismatch {
                expected: 2,
                actual: 1,
                ..
            }
        ));

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_clawback_v1_claim_rejects_v2() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(1000).await?;
        let mut bob = alice.next(0).await?;

        let timestamp = alice.new_block_with_current_time().await?;

        let coin_spends = alice
            .wallet
            .send_xch(
                vec![(bob.puzzle_hash, 1000)],
                0,
                vec![],
                Some(timestamp + 3600),
            )
            .await?;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;
        bob.wait_for_puzzles().await;

        let (rows, _) = alice
            .wallet
            .db
            .coin_records(
                AssetFilter::Id(Bytes32::default()),
                10,
                0,
                CoinSortMode::CreatedHeight,
                false,
                CoinFilterMode::Clawback,
            )
            .await?;
        let coin_id = rows[0].coin.coin_id();
        assert_eq!(rows[0].clawback_version, Some(2));

        let err = bob
            .wallet
            .claim_clawback(vec![coin_id], 0)
            .await
            .expect_err("claim on V2 should fail");
        assert!(matches!(
            err,
            WalletError::ClawbackVersionMismatch {
                expected: 1,
                actual: 2,
                ..
            }
        ));

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_clawback_v2_finalize() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(1000).await?;
        let mut bob = alice.next(0).await?;

        let timestamp = alice.new_block_with_current_time().await?;
        // Short absolute expiry so finalize is soon available.
        let expiry = timestamp + 2;

        let coin_spends = alice
            .wallet
            .send_xch(vec![(bob.puzzle_hash, 1000)], 0, vec![], Some(expiry))
            .await?;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;
        bob.wait_for_puzzles().await;

        let (rows, _) = alice
            .wallet
            .db
            .coin_records(
                AssetFilter::Id(Bytes32::default()),
                10,
                0,
                CoinSortMode::CreatedHeight,
                false,
                CoinFilterMode::Clawback,
            )
            .await?;
        let coin_id = rows[0].coin.coin_id();
        assert_eq!(rows[0].clawback_version, Some(2));

        sleep(Duration::from_secs(3)).await;
        alice.new_block_with_current_time().await?;

        let coin_spends = alice.wallet.finalize_clawback(vec![coin_id], 0).await?;
        assert!(!coin_spends.is_empty());
        alice.transact(coin_spends).await?;
        bob.wait_for_coins().await;

        assert_eq!(bob.wallet.db.selectable_xch_balance().await?, 1000);
        assert_eq!(alice.wallet.db.selectable_xch_balance().await?, 0);

        Ok(())
    }
}
