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
    driver::Puzzle,
    prelude::*,
    types::{
        Mod,
        puzzles::{
            AddDelegatedPuzzleWrapper, AddDelegatedPuzzleWrapperSolution,
            DelegatedPuzzleFeederSolution, Force1of2RestrictedVariable,
            Force1of2RestrictedVariableSolution, Timelock,
        },
    },
};
use sage_database::{VaultConfig, VaultRecoveryRow};

use crate::{recovery_state_hashes, vault_hashes};

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
