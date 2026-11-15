//! Recovering a vault, cancelling or finishing a recovery, and rekeying a vault.
//!
//! A vault's top level is a 1 of 2 of `[custody, recovery]`. The recovery side can only start
//! recovery, by recreating the vault in the recovery state: a 1 of 2 of the same custody and a
//! timelocked member that recreates the vault with its new custody. Until the timelock expires, the
//! custody can cancel the recovery by recreating the vault as it was. Afterwards, anyone can finish
//! it, since the finish member needs no signature. The custody can also rekey the vault at any time
//! outside of recovery, by recreating it with a new config.

#[cfg(test)]
mod tests;

use chia_wallet_sdk::{
    chia::puzzle_types::Memos,
    clvm_utils::TreeHash,
    driver::{InnerPuzzleSpend, MipsSpend, Vault, VaultVersion},
    prelude::*,
    types::{
        Mod,
        puzzles::{
            Force1of2RestrictedVariable, Force1of2RestrictedVariableSolution,
            ForceSingletonRecreationMod, PreventConditionOpcode, PreventMultipleCreateCoinsMod,
            Timelock,
        },
    },
};
use sage_database::{
    VaultConfig, VaultConfigRow, VaultLayout, VaultRecoveryConfig, VaultRecoveryRow, VaultSigners,
    VaultState,
};

use crate::{
    PREVENTED_RECOVERY_OPCODES, VaultHashes, WalletError, WalletKind, build_vault,
    build_vault_recovery, recovery_finish_restriction, recovery_restrictions, signers_hash,
    vault_hashes,
};

use super::{SpendableVault, Wallet, insert_signers_spend, insert_top_level_spend};

/// A recovery that's ready to be signed by the vault's recovery keys.
#[derive(Debug, Clone)]
pub struct VaultRecoveryStart {
    pub coin_spends: Vec<CoinSpend>,
    /// The vault coin whose spend starts the recovery.
    pub vault_coin: Coin,
    /// The recovery that the vault is in once the spend is confirmed.
    pub recovery: VaultRecoveryRow,
    /// The custody hash the vault moves to once the recovery finishes.
    pub post_recovery_custody_hash: Bytes32,
}

/// A spend that finishes a recovery, which needs no signature.
#[derive(Debug, Clone)]
pub struct VaultRecoveryFinish {
    pub coin_spends: Vec<CoinSpend>,
    /// The vault coin whose spend finishes the recovery.
    pub vault_coin: Coin,
    /// The vault's config once the recovery finishes.
    pub post_recovery: VaultConfig,
}

/// The recovery that a vault's current coin is in.
#[derive(Debug, Clone)]
pub struct PendingRecovery {
    /// The vault's current coin, in the recovery state.
    pub vault: Vault,
    /// The custody that started recovery, which can cancel it.
    pub custody: VaultSigners,
    /// The hashes of the recovery state, whose right side finishes recovery.
    pub hashes: VaultHashes,
    /// The custody hash of the vault before recovery started, which cancelling it recreates.
    pub pre_recovery_custody_hash: Bytes32,
    /// The config from before recovery started, if it's known, whose memo cancelling it reveals.
    pub pre_recovery: Option<VaultConfig>,
    pub recovery_timelock: u64,
    /// The recovery state's config and the vault's config once the recovery finishes, if the
    /// recovery's memo revealed them. A recovery can only be finished with them.
    pub finish: Option<(VaultRecoveryConfig, VaultConfig)>,
    /// When the current coin was created, from its block's timestamp, which the timelock is
    /// relative to.
    pub timelock_from: Option<u64>,
    /// Whether this wallet started the recovery.
    pub is_started_locally: bool,
}

impl PendingRecovery {
    /// When the recovery can be finished, from block timestamps, if the current coin's timestamp is
    /// known.
    pub fn finishes_at(&self) -> Option<u64> {
        self.timelock_from?.checked_add(self.recovery_timelock)
    }

    /// Whether the timelock has expired by the given time on chain.
    pub fn can_finish_at(&self, chain_timestamp: u64) -> bool {
        self.finishes_at()
            .is_some_and(|finishes_at| chain_timestamp >= finishes_at)
    }
}

impl Wallet {
    /// Starts a recovery of the vault with its recovery keys, which moves it to `post_recovery`
    /// once the timelock expires, unless the current custody cancels it first.
    ///
    /// The recovery side can't send messages or create other coins, so neither the vault nor its
    /// coins can pay a fee. It's paid by another wallet's coins, if at all.
    pub async fn start_vault_recovery(
        &self,
        post_recovery: &VaultConfig,
        recovery_keys: &[PublicKey],
    ) -> Result<VaultRecoveryStart, WalletError> {
        let (vault, config) = self.active_vault().await?;

        let mut ctx = SpendContext::new();
        let recovery = spend_start_recovery(
            &mut ctx,
            &vault,
            &config,
            post_recovery,
            recovery_keys,
            true,
        )?;

        Ok(VaultRecoveryStart {
            coin_spends: ctx.take(),
            vault_coin: vault.coin,
            recovery,
            post_recovery_custody_hash: vault_hashes(post_recovery)?.custody_hash.into(),
        })
    }

    /// The recovery that the vault's current coin is in, if it can be spent now.
    pub async fn pending_recovery(&self) -> Result<PendingRecovery, WalletError> {
        let current = self.current_vault().await?;

        let VaultState::RecoveryPending { timelock_from, .. } = current.status.state else {
            return Err(WalletError::VaultNoRecoveryPending);
        };

        let launcher_id = current.status.launcher_id;
        let custody_hash: Bytes32 = current.vault.info.custody_hash.into();

        let (pre_recovery_custody_hash, pre_recovery) = self
            .pre_recovery_config(launcher_id, current.vault.coin)
            .await?;

        let finish = match current.status.config.as_ref().map(|row| &row.layout) {
            Some(VaultLayout::Recovery(config)) => {
                match self.db.vault_config(config.recovery_custody_hash).await? {
                    Some(VaultConfigRow {
                        layout: VaultLayout::Custody(post_recovery),
                        ..
                    }) => Some((config.clone(), post_recovery)),
                    _ => None,
                }
            }
            _ => None,
        };

        let (custody, hashes, recovery_timelock) = if let Some((config, post_recovery)) = &finish {
            let mut ctx = SpendContext::new();
            let built = build_vault_recovery(&mut ctx, config, post_recovery)?;
            (
                config.custody.clone(),
                built.hashes,
                config.recovery_timelock,
            )
        } else if let Some(recovery) = current.status.recovery {
            let custody = pre_recovery
                .as_ref()
                .map(|config| config.custody.clone())
                .ok_or(WalletError::VaultConfigUnknown)?;

            let custody_member_hash = signers_hash(&custody, vec![])?;

            if custody_member_hash != TreeHash::from(recovery.custody_member_hash) {
                return Err(WalletError::VaultConfigUnknown);
            }

            let hashes = crate::recovery_state_hashes(
                custody_member_hash,
                recovery.recovery_timelock,
                recovery.finish_puzzle_hash,
            );

            (custody, hashes, recovery.recovery_timelock)
        } else {
            return Err(WalletError::VaultConfigUnknown);
        };

        if hashes.custody_hash != TreeHash::from(custody_hash) {
            return Err(WalletError::VaultConfigUnknown);
        }

        Ok(PendingRecovery {
            vault: current.vault,
            custody,
            hashes,
            pre_recovery_custody_hash,
            pre_recovery,
            recovery_timelock,
            finish,
            timelock_from,
            is_started_locally: current
                .status
                .recovery
                .is_some_and(|recovery| recovery.is_started_locally),
        })
    }

    /// The custody hash of the vault before its current recovery started, and its config if it's
    /// known. The custody can recreate the vault while recovery is pending, so the coins in the
    /// recovery state are skipped, back to the coin that started it.
    async fn pre_recovery_config(
        &self,
        launcher_id: Bytes32,
        coin: Coin,
    ) -> Result<(Bytes32, Option<VaultConfig>), WalletError> {
        let coins = self.db.vault_coins(launcher_id).await?;

        let index = coins
            .iter()
            .position(|row| row.coin == coin)
            .ok_or(WalletError::VaultNotSynced)?;

        let recovery_hash = coins[index].custody_hash;

        let started_by = coins[..index]
            .iter()
            .rev()
            .find(|row| row.custody_hash != recovery_hash)
            .and_then(|row| row.custody_hash)
            .ok_or(WalletError::VaultConfigUnknown)?;

        let config = match self.db.vault_config(started_by).await? {
            Some(VaultConfigRow {
                layout: VaultLayout::Custody(config),
                ..
            }) => Some(config),
            _ => None,
        };

        Ok((started_by, config))
    }

    /// Cancels the vault's pending recovery with its custody, by recreating it as it was before
    /// recovery started. The custody side isn't restricted, so the vault can pay the fee.
    pub async fn cancel_vault_recovery(&self, fee: u64) -> Result<Vec<CoinSpend>, WalletError> {
        let recovery = self.pending_recovery().await?;

        let mut ctx = SpendContext::new();

        let memo_list = match &recovery.pre_recovery {
            Some(config) => Some(build_vault(&mut ctx, config)?.memo_list),
            None => None,
        };

        let vault = SpendableVault {
            vault: recovery.vault,
            custody: recovery.custody,
            recovery_member_hash: recovery.hashes.recovery_member_hash,
            child_custody_hash: recovery.pre_recovery_custody_hash.into(),
            child_memo: memo_list,
        };

        self.spend_vault_with(&mut ctx, &vault, fee).await?;

        Ok(ctx.take())
    }

    /// Finishes the vault's pending recovery, which anyone can do once the timelock has expired.
    /// The finish member's conditions are fixed, so the vault can't pay a fee.
    ///
    /// It's refused before `chain_timestamp` reaches the end of the timelock, since the spend can't
    /// be confirmed until then.
    pub async fn finish_vault_recovery(
        &self,
        chain_timestamp: u64,
    ) -> Result<VaultRecoveryFinish, WalletError> {
        let recovery = self.pending_recovery().await?;

        let Some((config, post_recovery)) = recovery.finish.clone() else {
            return Err(WalletError::VaultRecoveryUnfinishable);
        };

        if !recovery.can_finish_at(chain_timestamp) {
            return Err(WalletError::VaultRecoveryTimelock(recovery.finishes_at()));
        }

        let mut ctx = SpendContext::new();
        spend_finish_recovery(&mut ctx, &recovery.vault, &config, &post_recovery)?;

        Ok(VaultRecoveryFinish {
            coin_spends: ctx.take(),
            vault_coin: recovery.vault.coin,
            post_recovery,
        })
    }

    /// Recreates the vault with a new config, such as a new custody or recovery key, a new
    /// recovery timelock, or a newer puzzle version. The new config is revealed in a memo.
    pub async fn rekey_vault(
        &self,
        new_config: &VaultConfig,
        fee: u64,
    ) -> Result<Vec<CoinSpend>, WalletError> {
        let (vault, config) = self.active_vault().await?;

        let mut ctx = SpendContext::new();

        let built = build_vault(&mut ctx, new_config)?;

        if built.hashes.custody_hash == vault.info.custody_hash {
            return Err(WalletError::VaultRekeyUnchanged);
        }

        let vault = SpendableVault::new(vault, &config)?
            .with_child(built.hashes.custody_hash, built.memo_list);

        self.spend_vault_with(&mut ctx, &vault, fee).await?;

        Ok(ctx.take())
    }

    /// Spends the vault with its custody, paying the fee from its XCH, and recreates it as the
    /// spendable vault says.
    async fn spend_vault_with(
        &self,
        ctx: &mut SpendContext,
        vault: &SpendableVault,
        fee: u64,
    ) -> Result<(), WalletError> {
        if fee == 0 {
            let mut conditions = Conditions::new();

            if let Some(expires_at) = self.transaction_expires_at()? {
                conditions = conditions.assert_before_seconds_absolute(expires_at);
            }

            return self.spend_vault(ctx, vault, conditions);
        }

        let actions = [Action::fee(fee)];
        let mut spends = Spends::new(self.change_p2_puzzle_hash().await?);

        self.select_spends_for_deltas(ctx, &mut spends, Deltas::from_actions(&actions), &[])
            .await?;

        let deltas = spends.apply(ctx, &actions)?;

        if let Some(expires_at) = self.transaction_expires_at()? {
            spends.conditions.required = spends
                .conditions
                .required
                .assert_before_seconds_absolute(expires_at);
        }

        self.complete_vault_spends_with(ctx, &deltas, spends, vault)
            .await?;

        Ok(())
    }

    /// Records a recovery that this wallet started, so that it isn't alerted about it.
    pub async fn record_started_recovery(
        &self,
        recovery: VaultRecoveryRow,
    ) -> Result<(), WalletError> {
        let mut tx = self.db.tx().await?;
        tx.insert_vault_recovery(VaultRecoveryRow {
            is_started_locally: true,
            ..recovery
        })
        .await?;
        tx.commit().await?;
        Ok(())
    }

    /// Pays a fee from this BLS wallet's XCH in a transaction that also spends `coin_id`, such as a
    /// vault recovery, which can't pay its own fee. The fee's spends assert that the coin is spent
    /// alongside them, so they can't be confirmed on their own.
    pub async fn pay_fee_alongside(
        &self,
        fee: u64,
        coin_id: Bytes32,
    ) -> Result<Vec<CoinSpend>, WalletError> {
        if !matches!(self.kind, WalletKind::Bls { .. }) {
            return Err(WalletError::VaultFeeWalletNotBls);
        }

        let mut ctx = SpendContext::new();
        let actions = [Action::fee(fee)];

        let mut spends = self.prepare_spends(&mut ctx, vec![], &actions).await?;
        spends.conditions.required = spends.conditions.required.assert_concurrent_spend(coin_id);

        let deltas = spends.apply(&mut ctx, &actions)?;
        self.complete_spends(&mut ctx, &deltas, spends).await?;

        Ok(ctx.take())
    }
}

/// Spends a vault through its recovery side to start a recovery, which moves it to
/// `post_recovery` once the timelock expires. Returns the recovery that the vault is in once the
/// spend is confirmed, as this wallet started it.
///
/// Members of the recovery side whose keys are in `recovery_keys` are preferred. The recovery's
/// memo reveals the vault's config once it finishes, and without it, the recovery can only be
/// cancelled. Sage and Cloud Wallet always reveal it, so `reveal_memo` is only false in tests of
/// recoveries started elsewhere.
pub fn spend_start_recovery(
    ctx: &mut SpendContext,
    vault: &Vault,
    config: &VaultConfig,
    post_recovery: &VaultConfig,
    recovery_keys: &[PublicKey],
    reveal_memo: bool,
) -> Result<VaultRecoveryRow, WalletError> {
    let recovery = config
        .recovery
        .as_ref()
        .ok_or(WalletError::VaultHasNoRecovery)?;

    let hashes = vault_hashes(config)?;
    let recovery_member_hash = hashes
        .recovery_member_hash
        .ok_or(WalletError::VaultHasNoRecovery)?;

    if hashes.custody_hash != vault.info.custody_hash {
        return Err(WalletError::VaultConfigUnknown);
    }

    let amount = vault.coin.amount;

    let recovery_config = VaultRecoveryConfig {
        custody: config.custody.clone(),
        recovery_timelock: recovery.timelock,
        version: recovery.version,
        recovery_custody_hash: vault_hashes(post_recovery)?.custody_hash.into(),
        amount,
    };

    let built = build_vault_recovery(ctx, &recovery_config, post_recovery)?;
    let recovery_custody_hash: Bytes32 = built.hashes.custody_hash.into();
    let finish_puzzle = built
        .recovery_finish_puzzle
        .ok_or(WalletError::VaultHasNoRecovery)?;
    let finish_puzzle_hash: Bytes32 = ctx.tree_hash(finish_puzzle).into();

    let memos = if reveal_memo {
        Memos::Some(built.memo_list)
    } else {
        Memos::None
    };

    // Force singleton recreation requires the vault to assert its amount, which it recreates.
    let conditions = Conditions::new()
        .create_coin(recovery_custody_hash, amount, memos)
        .assert_my_amount(amount);

    let mut spend = MipsSpend::new(ctx.delegated_spend(conditions)?);

    insert_top_level_spend(&mut spend, hashes.custody_member_hash, recovery_member_hash);
    insert_signers_spend(
        ctx,
        &mut spend,
        recovery_member_hash,
        recovery_restrictions(
            hashes.custody_member_hash,
            recovery.timelock,
            recovery.version,
        ),
        &recovery.signers,
        recovery_keys,
    )?;

    // The recovery side's delegated puzzle wrappers. The first forces the vault to be recreated in
    // the recovery state, with the finish member chosen in its solution.
    let force_1_of_2 = Force1of2RestrictedVariable::new(
        hashes.custody_member_hash.into(),
        0,
        vec![Timelock::new(recovery.timelock).curry_tree_hash()]
            .tree_hash()
            .into(),
        ().tree_hash().into(),
    );
    let puzzle = ctx.curry(force_1_of_2)?;
    let solution = ctx.alloc(&Force1of2RestrictedVariableSolution::new(
        finish_puzzle_hash,
    ))?;
    spend
        .restrictions
        .insert(force_1_of_2.curry_tree_hash(), Spend::new(puzzle, solution));

    for opcode in PREVENTED_RECOVERY_OPCODES {
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

    if recovery.version == VaultVersion::ForceSingletonRecreation {
        let puzzle = ctx.alloc_mod::<ForceSingletonRecreationMod>()?;
        spend.restrictions.insert(
            ForceSingletonRecreationMod::mod_hash(),
            Spend::new(puzzle, NodePtr::NIL),
        );
    }

    vault.spend(ctx, &spend)?;

    Ok(VaultRecoveryRow {
        custody_hash: recovery_custody_hash,
        custody_member_hash: hashes.custody_member_hash.into(),
        recovery_member_hash: built
            .hashes
            .recovery_member_hash
            .ok_or(WalletError::VaultHasNoRecovery)?
            .into(),
        finish_puzzle_hash,
        recovery_timelock: recovery.timelock,
        is_started_locally: true,
    })
}

/// Spends a vault in the recovery state through its timelocked finish member, which recreates it
/// with `post_recovery`. It needs no signature, but can only be confirmed once the timelock has
/// expired.
pub fn spend_finish_recovery(
    ctx: &mut SpendContext,
    vault: &Vault,
    config: &VaultRecoveryConfig,
    post_recovery: &VaultConfig,
) -> Result<(), WalletError> {
    let built = build_vault_recovery(ctx, config, post_recovery)?;

    if built.hashes.custody_hash != vault.info.custody_hash {
        return Err(WalletError::VaultRecoveryUnfinishable);
    }

    let finish_puzzle = built
        .recovery_finish_puzzle
        .ok_or(WalletError::VaultRecoveryUnfinishable)?;
    let recovery_member_hash = built
        .hashes
        .recovery_member_hash
        .ok_or(WalletError::VaultRecoveryUnfinishable)?;

    // The finish member outputs its fixed conditions, so the delegated puzzle outputs none.
    let mut spend = MipsSpend::new(ctx.delegated_spend(Conditions::new())?);

    insert_top_level_spend(
        &mut spend,
        built.hashes.custody_member_hash,
        recovery_member_hash,
    );

    let restriction = recovery_finish_restriction(config.recovery_timelock);

    spend.members.insert(
        recovery_member_hash,
        InnerPuzzleSpend::new(
            0,
            vec![restriction],
            Spend::new(finish_puzzle, NodePtr::NIL),
        ),
    );

    let timelock = ctx.curry(Timelock::new(config.recovery_timelock))?;
    spend
        .restrictions
        .insert(restriction.puzzle_hash, Spend::new(timelock, NodePtr::NIL));

    vault.spend(ctx, &spend)?;

    Ok(())
}
