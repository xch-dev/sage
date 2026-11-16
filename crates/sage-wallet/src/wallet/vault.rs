mod assets;
mod recovery;

#[cfg(test)]
mod asset_tests;
#[cfg(test)]
mod offer_tests;
#[cfg(test)]
mod tests;

pub use assets::*;
pub use recovery::*;

use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

use chia_wallet_sdk::{
    chia::puzzle_types::{EveProof, LineageProof, Memos, Proof, offer::SettlementPaymentsSolution},
    clvm_utils::TreeHash,
    driver::{
        ClawbackV2, Delta, Finished, InnerPuzzleSpend, MipsSpend, MofN, P2Singleton, Restriction,
        SpendKind, SpendableAsset, VAULT_AMOUNT, VAULT_MESSAGE_MODE, Vault, VaultInfo,
        mips_puzzle_hash,
    },
    prelude::*,
    types::puzzles::BlsMember,
};
use sage_database::{
    CoinKind, ConditionsOrSingleton, P2Puzzle, VaultConfig, VaultLayout, VaultMember, VaultSigners,
    VaultState, VaultStatus,
};

use crate::{
    VaultReveals, VaultSpendSummary, WalletError, WalletKind, build_vault, member_puzzle_hash,
    parse_vault_spend_bundle_with, signers_hash, signers_inner_hash, vault_hashes,
};

use super::Wallet;

/// The shortest recovery timelock Sage mints a vault with. A recovery that this wallet didn't start
/// has to be noticed and cancelled before it expires, so it can't be too short.
pub const MIN_RECOVERY_TIMELOCK: u64 = 60 * 60;

/// The hardened BLS derivation path purpose of vault custody keys, which are derived from a
/// wallet's master key as `m/12381'/8444'/12'/index'`. The other purposes Chia uses are 0 to 6, so
/// this doesn't overlap with them. Deriving the custody key means the mnemonic the user already
/// backed up also backs it up.
pub const VAULT_CUSTODY_KEY_PURPOSE: u32 = 12;

/// Derives a vault custody key from a wallet's master secret key.
pub fn vault_custody_key(master_sk: &SecretKey, index: u32) -> SecretKey {
    master_sk
        .derive_hardened(12381)
        .derive_hardened(8444)
        .derive_hardened(VAULT_CUSTODY_KEY_PURPOSE)
        .derive_hardened(index)
}

/// A vault mint, which is confirmed once the transaction that creates its launcher is.
#[derive(Debug, Clone)]
pub struct VaultMint {
    pub launcher_id: Bytes32,
    pub coin_spends: Vec<CoinSpend>,
}

/// A vault's current coin, the custody that spends it, and what it's recreated as.
#[derive(Debug, Clone)]
pub struct SpendableVault {
    pub vault: Vault,
    /// The signers of the custody side, which spend the vault.
    pub custody: VaultSigners,
    /// The right side of the vault's top level 1 of 2, or `None` if its custody is the top level.
    pub recovery_member_hash: Option<TreeHash>,
    /// The custody hash the vault is recreated with.
    pub child_custody_hash: TreeHash,
    /// The CHIP-0043 memo list the vault is recreated with, which is only needed when the custody
    /// changes, so that it can be followed without the config.
    pub child_memo: Option<NodePtr>,
}

impl SpendableVault {
    /// A vault in its normal state with the config, which is recreated with the same custody.
    pub fn new(vault: Vault, config: &VaultConfig) -> Result<Self, WalletError> {
        let hashes = vault_hashes(config)?;

        Ok(Self {
            vault,
            custody: config.custody.clone(),
            recovery_member_hash: hashes.recovery_member_hash,
            child_custody_hash: hashes.custody_hash,
            child_memo: None,
        })
    }

    /// Recreates the vault with another custody, along with its memo.
    #[must_use]
    pub fn with_child(mut self, custody_hash: TreeHash, memo_list: NodePtr) -> Self {
        self.child_custody_hash = custody_hash;
        self.child_memo = Some(memo_list);
        self
    }

    /// Whether the spend changes the vault's custody.
    pub fn changes_custody(&self) -> bool {
        self.child_custody_hash != self.vault.info.custody_hash
    }
}

/// A vault's latest coin, which can be spent now.
#[derive(Debug, Clone)]
pub struct CurrentVault {
    pub vault: Vault,
    pub status: VaultStatus,
}

impl Wallet {
    /// The launcher id of the vault, if this is a vault wallet.
    pub fn vault_launcher_id(&self) -> Option<Bytes32> {
        match self.kind {
            WalletKind::Vault { launcher_id } => Some(launcher_id),
            WalletKind::Bls { .. } | WalletKind::Watch => None,
        }
    }

    /// Mints a vault with the config from this wallet's XCH, with its CHIP-0043 memo in the launcher
    /// solution, so that it can be imported from its launcher id alone.
    pub async fn mint_vault(
        &self,
        config: &VaultConfig,
        fee: u64,
    ) -> Result<VaultMint, WalletError> {
        if !matches!(self.kind, WalletKind::Bls { .. }) {
            return Err(WalletError::VaultMintNotSupported);
        }

        let mut ctx = SpendContext::new();
        let built = build_vault(&mut ctx, config)?;

        let actions = [Action::fee(fee)];
        let mut deltas = Deltas::from_actions(&actions);
        *deltas.update(Id::Xch) += Delta::new(0, VAULT_AMOUNT);
        deltas.set_needed(Id::Xch);

        let mut spends = self.prepare_spends_for_selection(&mut ctx, &[]).await?;
        self.select_spends_for_deltas(&mut ctx, &mut spends, deltas.clone(), &[])
            .await?;
        spends.apply(&mut ctx, &actions)?;

        let (source, launcher) = spends.xch.create_launcher(VAULT_AMOUNT)?;
        let (parent_conditions, vault) =
            launcher.mint_vault(&mut ctx, built.hashes.custody_hash, built.memo_list)?;

        match &mut spends.xch.items[source].kind {
            SpendKind::Conditions(spend) => spend.add_conditions(parent_conditions),
            SpendKind::Settlement(_) => return Err(DriverError::CannotEmitConditions.into()),
        }

        self.complete_spends(&mut ctx, &deltas, spends).await?;

        Ok(VaultMint {
            launcher_id: vault.info.launcher_id,
            coin_spends: ctx.take(),
        })
    }

    /// What the wallet knows about the coins a transaction spends, which clear signing needs to
    /// parse the vault's spend: the clawbacks and P2 conditions or singleton puzzles they're
    /// locked to, and the nonces of the vault's addresses.
    pub async fn vault_reveals(
        &self,
        coin_spends: &[CoinSpend],
    ) -> Result<VaultReveals, WalletError> {
        let mut reveals = VaultReveals::default();

        let Some(launcher_id) = self.vault_launcher_id() else {
            return Ok(reveals);
        };

        let coin_ids: Vec<Bytes32> = coin_spends.iter().map(|cs| cs.coin.coin_id()).collect();

        for row in self.db.coins_by_ids(&coin_ids).await? {
            match self.db.p2_puzzle(row.p2_puzzle_hash).await {
                Ok(P2Puzzle::Clawback(clawback)) if clawback.version == 2 => {
                    reveals.clawbacks.push(ClawbackV2::new(
                        clawback.sender_puzzle_hash,
                        clawback.receiver_puzzle_hash,
                        clawback.seconds,
                        row.coin.amount,
                        row.kind != CoinKind::Xch,
                    ));
                }
                Ok(P2Puzzle::P2ConditionsOrSingleton(p2)) => {
                    reveals.p2_conditions_or_singletons.push(p2.p2);
                }
                _ => {}
            }
        }

        let address_count = self.db.p2_singleton_puzzle_hashes(launcher_id).await?.len();
        reveals.nonces = (0..address_count.max(1)).collect();

        Ok(reveals)
    }

    /// Parses what the vault's spend in a transaction does with clear signing, if it's spent.
    pub async fn parse_vault_spend(
        &self,
        coin_spends: &[CoinSpend],
    ) -> Result<Option<VaultSpendSummary>, WalletError> {
        let Some(launcher_id) = self.vault_launcher_id() else {
            return Ok(None);
        };

        let reveals = self.vault_reveals(coin_spends).await?;

        parse_vault_spend_bundle_with(coin_spends, launcher_id, &reveals)
    }

    /// The vault's current coin and config, if it can be spent now.
    ///
    /// The vault's custody members sign over its coin id, so a vault can only have one
    /// unconfirmed spend at a time. A spend made while another is pending would spend the same
    /// coin, so it's refused until the pending one confirms.
    pub async fn spendable_vault(&self) -> Result<SpendableVault, WalletError> {
        let (vault, config) = self.active_vault().await?;
        SpendableVault::new(vault, &config)
    }

    /// The vault's current coin and config, if it's active and can be spent now.
    pub async fn active_vault(&self) -> Result<(Vault, VaultConfig), WalletError> {
        let current = self.current_vault().await?;

        if let VaultState::RecoveryPending { .. } = current.status.state {
            return Err(WalletError::VaultRecoveryPending);
        }

        let config = match current.status.config.map(|config| config.layout) {
            Some(VaultLayout::Custody(config)) => config,
            Some(VaultLayout::Recovery(_)) => return Err(WalletError::VaultRecoveryPending),
            None => return Err(WalletError::VaultConfigUnknown),
        };

        if vault_hashes(&config)?.custody_hash != current.vault.info.custody_hash {
            return Err(WalletError::VaultConfigUnknown);
        }

        Ok((current.vault, config))
    }

    /// The vault's latest coin, if it's confirmed and doesn't have a pending spend, so that it can
    /// be spent now. Its custody hash must be known, but its config doesn't have to be.
    ///
    /// The vault's custody members sign over its coin id, so a vault can only have one
    /// unconfirmed spend at a time. A spend made while another is pending would spend the same
    /// coin, so it's refused until the pending one confirms.
    pub async fn current_vault(&self) -> Result<CurrentVault, WalletError> {
        self.latest_vault(true).await
    }

    /// The vault's latest coin, which must be spendable now if `spendable` is true. Otherwise, it
    /// can be unconfirmed or have a pending spend, which is enough to describe it.
    async fn latest_vault(&self, spendable: bool) -> Result<CurrentVault, WalletError> {
        let Some(launcher_id) = self.vault_launcher_id() else {
            return Err(WalletError::VaultSpendNotSupported);
        };

        let status = self
            .db
            .vault_status(launcher_id)
            .await?
            .ok_or(WalletError::VaultNotSynced)?;

        match status.state {
            VaultState::Unsynced => return Err(WalletError::VaultNotSynced),
            VaultState::Melted => return Err(WalletError::VaultMelted),
            VaultState::RecoveryPending { .. } | VaultState::Active => {}
        }

        let Some(row) = status.coin else {
            return Err(WalletError::VaultNotSynced);
        };

        if spendable && (row.created_height.is_none() || row.spent_height.is_some()) {
            return Err(WalletError::VaultSpendPending);
        }

        if spendable && self.db.is_coin_spend_pending(row.coin.coin_id()).await? {
            return Err(WalletError::VaultSpendPending);
        }

        let custody_hash = row.custody_hash.ok_or(WalletError::VaultConfigUnknown)?;

        let proof = self.vault_lineage_proof(launcher_id, row.coin).await?;

        Ok(CurrentVault {
            vault: Vault::new(
                row.coin,
                proof,
                VaultInfo::new(launcher_id, custody_hash.into()),
            ),
            status,
        })
    }

    /// The lineage proof of a vault coin, from its parent in the vault's lineage, or from its
    /// launcher if it's the eve coin.
    async fn vault_lineage_proof(
        &self,
        launcher_id: Bytes32,
        coin: Coin,
    ) -> Result<Proof, WalletError> {
        if coin.parent_coin_info == launcher_id {
            let launcher = self
                .db
                .coin_by_id(launcher_id)
                .await?
                .ok_or(WalletError::MissingCoin(launcher_id))?;

            return Ok(Proof::Eve(EveProof {
                parent_parent_coin_info: launcher.parent_coin_info,
                parent_amount: launcher.amount,
            }));
        }

        let parent = self
            .db
            .vault_coins(launcher_id)
            .await?
            .into_iter()
            .find(|row| row.coin.coin_id() == coin.parent_coin_info)
            .ok_or(WalletError::MissingCoin(coin.parent_coin_info))?;

        let parent_inner_puzzle_hash = parent
            .custody_hash
            .ok_or(WalletError::MissingCoin(coin.parent_coin_info))?;

        Ok(Proof::Lineage(LineageProof {
            parent_parent_coin_info: parent.coin.parent_coin_info,
            parent_inner_puzzle_hash,
            parent_amount: parent.coin.amount,
        }))
    }

    /// Spends the vault's coins with the conditions that the action system decided on. Each coin
    /// outputs its conditions through a delegated spend, which the vault approves by sending it a
    /// message, and the vault is recreated with the same custody. Conditions that no coin could
    /// output, such as the fee of a transaction that only spends the vault, are output by the
    /// vault itself, along with the extra conditions.
    pub(super) fn spend_vault_coins(
        &self,
        ctx: &mut SpendContext,
        spends: Spends<Finished>,
        p2_puzzles: &HashMap<Bytes32, P2Puzzle>,
        vault: &SpendableVault,
        extra_conditions: Conditions,
    ) -> Result<Outputs, WalletError> {
        let mut vault_conditions = spends.unplaced_conditions.clone();
        let mut coin_spends = HashMap::new();
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let launcher_id = vault.vault.info.launcher_id;
        let custody_hash: Bytes32 = vault.vault.info.custody_hash.into();
        let amount = vault.vault.coin.amount;

        for (asset, kind) in spends.unspent() {
            let spend = match kind {
                SpendKind::Conditions(spend) => {
                    let mut conditions = spend.finish();

                    let p2_puzzle = p2_puzzles.get(&asset.p2_puzzle_hash());

                    // A coin locked to fixed conditions or the vault, such as an offer's pre-split
                    // coin, asserts its own id when the vault spends it. Otherwise its delegated
                    // spend could be swapped onto the fixed path of another coin.
                    if matches!(p2_puzzle, Some(P2Puzzle::P2ConditionsOrSingleton(_))) {
                        conditions = conditions.assert_my_coin_id(asset.coin().coin_id());
                    }

                    // A clawback is spent by whichever side the vault is, through its address.
                    // The vault asserts that the path is open, so clear signing can tell which
                    // path the clawback takes. Only the vault's own conditions are trusted for
                    // that, rather than the clawback's.
                    let clawback = match p2_puzzle {
                        Some(P2Puzzle::Clawback(clawback)) if clawback.version == 2 => {
                            let sender =
                                vault_address(p2_puzzles, clawback.sender_puzzle_hash, launcher_id);
                            let receiver = vault_address(
                                p2_puzzles,
                                clawback.receiver_puzzle_hash,
                                launcher_id,
                            );

                            let v2 = ClawbackV2::new(
                                clawback.sender_puzzle_hash,
                                clawback.receiver_puzzle_hash,
                                clawback.seconds,
                                asset.coin().amount,
                                !matches!(asset, SpendableAsset::Xch(..)),
                            );

                            if let Some(p2) = sender
                                && timestamp < clawback.seconds
                            {
                                vault_conditions = vault_conditions
                                    .assert_before_seconds_absolute(clawback.seconds);
                                Some((v2, p2, true))
                            } else if let Some(p2) = receiver
                                && timestamp >= clawback.seconds
                            {
                                vault_conditions =
                                    vault_conditions.assert_seconds_absolute(clawback.seconds);
                                Some((v2, p2, false))
                            } else if sender.is_some() || receiver.is_some() {
                                return Err(WalletError::ClawbackSpendNotAllowed);
                            } else {
                                return Err(DriverError::MissingKey.into());
                            }
                        }
                        _ => None,
                    };

                    let delegated_spend = ctx.delegated_spend(conditions)?;
                    let delegated_puzzle_hash = ctx.tree_hash(delegated_spend.puzzle);
                    let coin_id = ctx.alloc(&asset.coin().coin_id())?;

                    vault_conditions = vault_conditions.send_message(
                        VAULT_MESSAGE_MODE,
                        delegated_puzzle_hash.to_vec().into(),
                        vec![coin_id],
                    );

                    if let Some((clawback, p2, is_sender)) = clawback {
                        let inner = p2.spend(ctx, custody_hash, amount, delegated_spend)?;

                        if is_sender {
                            clawback.sender_spend(ctx, inner)?
                        } else {
                            clawback.receiver_spend(ctx, inner)?
                        }
                    } else {
                        match p2_puzzle {
                            Some(P2Puzzle::P2Singleton(p2)) if p2.launcher_id == launcher_id => {
                                p2.spend(ctx, custody_hash, amount, delegated_spend)?
                            }
                            Some(P2Puzzle::P2ConditionsOrSingleton(ConditionsOrSingleton {
                                p2,
                                ..
                            })) if p2.launcher_id == launcher_id => {
                                p2.p2_singleton_spend(ctx, custody_hash, amount, delegated_spend)?
                            }
                            Some(P2Puzzle::External) => return Err(WalletError::WatchOnly),
                            _ => return Err(DriverError::MissingKey.into()),
                        }
                    }
                }
                SpendKind::Settlement(spend) => SettlementLayer
                    .construct_spend(ctx, SettlementPaymentsSolution::new(spend.finish()))?,
            };

            coin_spends.insert(asset.coin().coin_id(), spend);
        }

        let outputs = spends.spend(ctx, coin_spends)?;

        // A CAT whose TAIL the vault controls is only issued or melted with the vault's message.
        let pending = ctx.take();
        let tail_messages = vault_tail_messages(ctx, launcher_id, &pending)?;

        for coin_spend in pending {
            ctx.insert(coin_spend);
        }

        for message in tail_messages {
            vault_conditions.push(message);
        }

        self.spend_vault(ctx, vault, vault_conditions.extend(extra_conditions))?;

        Ok(outputs)
    }

    /// Spends the vault singleton with its custody, outputting the conditions and recreating it
    /// with its child custody and the same amount.
    pub fn spend_vault(
        &self,
        ctx: &mut SpendContext,
        vault: &SpendableVault,
        conditions: Conditions,
    ) -> Result<(), WalletError> {
        spend_vault_custody(ctx, vault, conditions, &self.vault_signer_keys())
    }
}

/// Spends a vault singleton with its custody, outputting the conditions and recreating it with its
/// child custody and the same amount. Members whose keys are in `signer_keys` are preferred.
pub fn spend_vault_custody(
    ctx: &mut SpendContext,
    vault: &SpendableVault,
    conditions: Conditions,
    signer_keys: &[PublicKey],
) -> Result<(), WalletError> {
    let amount = vault.vault.coin.amount;

    let memos = match vault.child_memo {
        Some(memo_list) => Memos::Some(memo_list),
        None => Memos::None,
    };

    let conditions = conditions
        .create_coin(vault.child_custody_hash.into(), amount, memos)
        .assert_my_amount(amount);

    let delegated_spend = ctx.delegated_spend(conditions)?;

    let mips_spend = custody_mips_spend(
        ctx,
        &vault.custody,
        vault.recovery_member_hash,
        delegated_spend,
        signer_keys,
    )?;

    vault.vault.spend(ctx, &mips_spend)?;

    Ok(())
}

/// The vault address with the puzzle hash, if it's one of this vault's.
fn vault_address(
    p2_puzzles: &HashMap<Bytes32, P2Puzzle>,
    puzzle_hash: Bytes32,
    launcher_id: Bytes32,
) -> Option<P2Singleton> {
    match p2_puzzles.get(&puzzle_hash) {
        Some(P2Puzzle::P2Singleton(p2)) if p2.launcher_id == launcher_id => Some(*p2),
        _ => None,
    }
}

/// The spend of a vault's custody puzzle down its custody side, with the delegated spend.
///
/// If the vault has a recovery side, its top level is a 1 of 2 of `[custody, recovery]`, whose
/// right side is `recovery_member_hash`. Otherwise its custody is the top level.
///
/// Members whose keys are in `signer_keys` are preferred, so that a vault with more custody members
/// than its threshold is signed by the ones this wallet holds.
pub fn custody_mips_spend(
    ctx: &mut SpendContext,
    custody: &VaultSigners,
    recovery_member_hash: Option<TreeHash>,
    delegated_spend: Spend,
    signer_keys: &[PublicKey],
) -> Result<MipsSpend, WalletError> {
    let mut spend = MipsSpend::new(delegated_spend);

    let side_hash = if let Some(recovery_member_hash) = recovery_member_hash {
        let custody_member_hash = signers_hash(custody, vec![])?;
        insert_top_level_spend(&mut spend, custody_member_hash, recovery_member_hash);
        custody_member_hash
    } else {
        mips_puzzle_hash(0, vec![], signers_inner_hash(custody), true)
    };

    insert_signers_spend(ctx, &mut spend, side_hash, vec![], custody, signer_keys)?;

    Ok(spend)
}

/// Inserts the spend of a vault's top level 1 of 2 of `[custody, recovery]`, which is spent through
/// either side.
pub(crate) fn insert_top_level_spend(
    spend: &mut MipsSpend,
    custody_member_hash: TreeHash,
    recovery_member_hash: TreeHash,
) {
    let items = vec![custody_member_hash, recovery_member_hash];

    let custody_hash = mips_puzzle_hash(
        0,
        vec![],
        MofN::new(1, items.clone()).inner_puzzle_hash(),
        true,
    );

    spend
        .members
        .insert(custody_hash, InnerPuzzleSpend::m_of_n(0, vec![], 1, items));
}

/// Inserts the spend of one side of a vault, which is a single member or an M of N of members, with
/// the restrictions on the side. Members whose keys are in `signer_keys` are preferred.
pub(crate) fn insert_signers_spend(
    ctx: &mut SpendContext,
    spend: &mut MipsSpend,
    side_hash: TreeHash,
    restrictions: Vec<Restriction>,
    signers: &VaultSigners,
    signer_keys: &[PublicKey],
) -> Result<(), WalletError> {
    if let [member] = signers.members.as_slice() {
        let member_spend = member_spend(ctx, *member)?;
        spend.members.insert(
            side_hash,
            InnerPuzzleSpend::new(0, restrictions, member_spend),
        );
        return Ok(());
    }

    let mut member_hashes: Vec<(TreeHash, VaultMember)> = signers
        .members
        .iter()
        .map(|&member| {
            (
                mips_puzzle_hash(0, vec![], member_puzzle_hash(member), false),
                member,
            )
        })
        .collect();

    member_hashes.sort_by_key(|(hash, _)| *hash);

    spend.members.insert(
        side_hash,
        InnerPuzzleSpend::m_of_n(
            0,
            restrictions,
            signers.threshold,
            member_hashes.iter().map(|(hash, _)| *hash).collect(),
        ),
    );

    // The members whose keys are held come first, then the others which could be signed for
    // elsewhere.
    let mut selected: Vec<&(TreeHash, VaultMember)> = member_hashes
        .iter()
        .filter(|(_, member)| is_signer(*member, signer_keys))
        .collect();

    selected.extend(
        member_hashes.iter().filter(|(_, member)| {
            !is_signer(*member, signer_keys) && member_spend_supported(*member)
        }),
    );

    if selected.len() < signers.threshold {
        return Err(WalletError::VaultMemberNotSupported);
    }

    for &(hash, member) in selected.into_iter().take(signers.threshold) {
        let member_spend = member_spend(ctx, member)?;
        spend
            .members
            .insert(hash, InnerPuzzleSpend::new(0, vec![], member_spend));
    }

    Ok(())
}

fn is_signer(member: VaultMember, signer_keys: &[PublicKey]) -> bool {
    matches!(
        member,
        VaultMember::Bls { public_key, fast_forward: false } if signer_keys.contains(&public_key)
    )
}

fn member_spend_supported(member: VaultMember) -> bool {
    matches!(
        member,
        VaultMember::Bls {
            fast_forward: false,
            ..
        }
    )
}

/// The spend of a custody member. Only BLS members which sign over the vault's coin id are
/// supported, and they sign the delegated puzzle hash with `AGG_SIG_ME`, so their solution is nil.
fn member_spend(ctx: &mut SpendContext, member: VaultMember) -> Result<Spend, WalletError> {
    match member {
        VaultMember::Bls {
            public_key,
            fast_forward: false,
        } => {
            let puzzle = ctx.curry(BlsMember::new(public_key))?;
            Ok(Spend::new(puzzle, NodePtr::NIL))
        }
        _ => Err(WalletError::VaultMemberNotSupported),
    }
}
