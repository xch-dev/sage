//! Recognizing vault recoveries from the spends that start them.
//!
//! A recovery is started through the recovery side of a vault's top level 1 of 2, whose
//! `force_1_of_2_restricted_variable` wrapper forces the vault to be recreated as a 1 of 2 of its
//! custody and a timelocked member that finishes recovery. The member is chosen in the wrapper's
//! solution, so the spend reveals it even if its memo is left out, as ent-wallet's
//! `parseRecoveryFinishMemberPuzzleHash` reads it. A recovery found this way can always be
//! cancelled by the custody, even though it can only be finished with the memo of its new custody.

use chia_wallet_sdk::{
    clvm_traits::FromClvm,
    clvm_utils::{ToTreeHash, TreeHash},
    clvmr::{Allocator, NodePtr},
    driver::{Layer, Puzzle, SingletonLayer},
    prelude::*,
    types::{
        Condition, Mod,
        puzzles::{
            AddDelegatedPuzzleWrapper, AddDelegatedPuzzleWrapperSolution,
            DelegatedPuzzleFeederSolution, Force1of2RestrictedVariable,
            Force1of2RestrictedVariableSolution, Timelock,
        },
        run_puzzle,
    },
};
use sage_database::{VaultConfig, VaultRecoveryRow};

use crate::{WalletError, parse_custody_reveal, recovery_state_hashes, vault_hashes};

/// What the spend that starts a vault's recovery reveals about the recovery state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecoveryStart {
    /// The tree hash of the hash of the custody side, which the recovery state keeps.
    pub left_side_subtree_hash_hash: Bytes32,
    /// The tree hash of the timelock restriction list of the member that finishes recovery.
    pub member_validator_list_hash: Bytes32,
    /// The puzzle hash of the member that finishes recovery.
    pub finish_puzzle_hash: Bytes32,
}

/// Parses the start of a recovery from the inner solution of a vault singleton's spend, if it was
/// spent through its recovery side. The outermost wrapper of the recovery side's delegated puzzle is
/// `force_1_of_2_restricted_variable`, whose solution is the finish member's puzzle hash.
pub fn parse_recovery_start(
    allocator: &Allocator,
    inner_solution: NodePtr,
) -> Option<RecoveryStart> {
    // The index wrapper passes its solution through to the delegated puzzle feeder.
    let feeder = DelegatedPuzzleFeederSolution::<NodePtr, NodePtr, NodePtr>::from_clvm(
        allocator,
        inner_solution,
    )
    .ok()?;

    let delegated = Puzzle::parse(allocator, feeder.delegated_puzzle).as_curried()?;

    if delegated.mod_hash != AddDelegatedPuzzleWrapper::<NodePtr, NodePtr>::mod_hash() {
        return None;
    }

    let wrapped =
        AddDelegatedPuzzleWrapper::<NodePtr, NodePtr>::from_clvm(allocator, delegated.args).ok()?;
    let wrapper = Puzzle::parse(allocator, wrapped.wrapper).as_curried()?;

    if wrapper.mod_hash != Force1of2RestrictedVariable::mod_hash() {
        return None;
    }

    let args = Force1of2RestrictedVariable::from_clvm(allocator, wrapper.args).ok()?;

    // Cloud Wallet's recovery state has no delegated puzzle validators on the finish member, and
    // every nonce is 0.
    if args.nonce != 0 || args.delegated_puzzle_validator_list_hash != ().tree_hash().into() {
        return None;
    }

    let solution = AddDelegatedPuzzleWrapperSolution::<NodePtr, NodePtr>::from_clvm(
        allocator,
        feeder.delegated_solution,
    )
    .ok()?;
    let solution =
        Force1of2RestrictedVariableSolution::from_clvm(allocator, solution.wrapper_solution)
            .ok()?;

    Some(RecoveryStart {
        left_side_subtree_hash_hash: args.left_side_subtree_hash_hash,
        member_validator_list_hash: args.member_validator_list_hash,
        finish_puzzle_hash: solution.new_right_side_member_hash,
    })
}

/// The recovery that a vault coin with `custody_hash` is in, if the spend of its parent with the
/// config `pre_recovery` started it. The recovery is only recognized if its recovery state rebuilds
/// the coin's custody hash, with the custody and timelock of the config.
pub fn observed_recovery(
    start: &RecoveryStart,
    pre_recovery: &VaultConfig,
    custody_hash: Bytes32,
) -> Option<VaultRecoveryRow> {
    let recovery = pre_recovery.recovery.as_ref()?;
    let custody_member_hash = vault_hashes(pre_recovery).ok()?.custody_member_hash;

    if Bytes32::from(custody_member_hash).tree_hash() != start.left_side_subtree_hash_hash.into() {
        return None;
    }

    let validators: Vec<TreeHash> = vec![Timelock::new(recovery.timelock).curry_tree_hash()];

    if validators.tree_hash() != start.member_validator_list_hash.into() {
        return None;
    }

    let hashes = recovery_state_hashes(
        custody_member_hash,
        recovery.timelock,
        start.finish_puzzle_hash,
    );

    if hashes.custody_hash != custody_hash.into() {
        return None;
    }

    Some(VaultRecoveryRow {
        custody_hash,
        custody_member_hash: custody_member_hash.into(),
        recovery_member_hash: hashes.recovery_member_hash?.into(),
        finish_puzzle_hash: start.finish_puzzle_hash,
        recovery_timelock: recovery.timelock,
        is_started_locally: false,
    })
}

/// Which recovery a spend through a vault's recovery side makes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VaultRecoverySpendKind {
    /// It starts a recovery, through the recovery keys.
    Start,
    /// It finishes a recovery, through the timelocked finish member.
    Finish,
}

/// A spend through a vault's recovery side, which clear signing can't parse, since the delegated
/// puzzle of a recovery start is wrapped, and a recovery finish has none.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VaultRecoverySpend {
    pub coin: Coin,
    /// The vault's custody hash before the spend.
    pub custody_hash: Bytes32,
    /// The custody hash the vault is recreated with, from running its puzzle, or `None` if it isn't.
    pub child_custody_hash: Option<Bytes32>,
    pub kind: VaultRecoverySpendKind,
}

/// Finds a spend of the vault with the launcher id through its recovery side. Returns `None` if the
/// vault isn't spent, or is spent through its custody, which reveals its signers.
pub fn parse_vault_recovery_spend(
    coin_spends: &[CoinSpend],
    launcher_id: Bytes32,
) -> Result<Option<VaultRecoverySpend>, WalletError> {
    let mut allocator = Allocator::new();

    for coin_spend in coin_spends {
        let puzzle = coin_spend.puzzle_reveal.to_clvm(&mut allocator)?;
        let puzzle = Puzzle::parse(&allocator, puzzle);

        let Some(singleton) = SingletonLayer::<Puzzle>::parse_puzzle(&allocator, puzzle)? else {
            continue;
        };

        if singleton.launcher_id != launcher_id {
            continue;
        }

        let solution = coin_spend.solution.to_clvm(&mut allocator)?;
        let solution = SingletonLayer::<Puzzle>::parse_solution(&allocator, solution)?;

        if parse_custody_reveal(
            &allocator,
            singleton.inner_puzzle.ptr(),
            solution.inner_solution,
        )
        .is_some()
        {
            return Ok(None);
        }

        let kind = if parse_recovery_start(&allocator, solution.inner_solution).is_some() {
            VaultRecoverySpendKind::Start
        } else {
            VaultRecoverySpendKind::Finish
        };

        let output = run_puzzle(
            &mut allocator,
            singleton.inner_puzzle.ptr(),
            solution.inner_solution,
        )?;
        let conditions = Vec::<Condition>::from_clvm(&allocator, output)?;

        // The singleton layer only allows one odd child, which is the next vault coin.
        let child_custody_hash = conditions
            .into_iter()
            .filter_map(Condition::into_create_coin)
            .find(|create_coin| create_coin.amount % 2 == 1)
            .map(|create_coin| create_coin.puzzle_hash);

        return Ok(Some(VaultRecoverySpend {
            coin: coin_spend.coin,
            custody_hash: singleton.inner_puzzle.curried_puzzle_hash().into(),
            child_custody_hash,
            kind,
        }));
    }

    Ok(None)
}
