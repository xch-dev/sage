//! Rebuilding the config of a vault whose spends don't carry CHIP-0043 memos, from what its spends
//! reveal and its recovery key.
//!
//! A spend through custody reveals the custody side of the top level 1 of 2: the custody member, or
//! the M of N and the members that signed. The recovery side only appears as a hash, which depends
//! on the recovery member, the recovery timelock, the puzzle version, and the custody side. So given
//! the recovery key, the timelock and version are searched for until the vault's custody hash is
//! rebuilt.

#[cfg(test)]
mod tests;

use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use chia_wallet_sdk::{
    chia::puzzle_types::singleton::SingletonArgs,
    clvm_utils::{ToTreeHash, TreeHash, tree_hash, tree_hash_atom, tree_hash_pair},
    clvmr::SExp,
    driver::{Layer, MofN, SingletonLayer, VaultVersion},
    prelude::*,
    puzzles::{DELEGATED_PUZZLE_FEEDER_HASH, RESTRICTIONS_HASH},
    types::{
        Mod,
        puzzles::{
            BlsMember, BlsMemberPuzzleAssert, DelegatedPuzzleFeederArgs,
            DelegatedPuzzleFeederSolution, EnforceDelegatedPuzzleWrappers,
            Force1of2RestrictedVariable, INDEX_WRAPPER_HASH, IndexWrapperArgs, K1Member,
            K1MemberPuzzleAssert, MofNArgs, MofNSolution, NofNArgs, OneOfNArgs, OneOfNSolution,
            PasskeyMember, PasskeyMemberPuzzleAssert, R1Member, R1MemberPuzzleAssert,
            SingletonMember, SingletonMemberWithMode, Timelock,
        },
    },
};
use sage_database::{
    VaultConfig, VaultConfigRow, VaultConfigSource, VaultLayout, VaultMember, VaultSigners,
};
use thiserror::Error;

use crate::{
    FAST_FORWARD_SINGLETON_MODE, WalletError, WalletPeer, fetch_vault_lineage, member_puzzle_hash,
    recovery_restrictions, signers_hash, vault_hashes,
};

/// Cloud Wallet's default recovery timelock, which is tried first.
pub const DEFAULT_RECOVERY_TIMELOCK: u64 = 12 * HOUR;

/// How many of a vault's spends are checked, from the latest, for the custody they reveal.
pub const MAX_VAULT_SPENDS_CHECKED: usize = 100;

/// How deeply an M of N proof is followed, which is far more than any vault's members need.
const MAX_PROOF_DEPTH: usize = 64;

const MINUTE: u64 = 60;
const HOUR: u64 = 60 * MINUTE;
const DAY: u64 = 24 * HOUR;

#[derive(Debug, Error)]
pub enum VaultHistoryError {
    #[error("No vault with this launcher id was found on chain")]
    NotFound,

    #[error(
        "The vault hasn't been spent through its custody yet, so its config can't be rebuilt from its spends"
    )]
    NoCustodySpends,

    #[error(
        "The recovery key doesn't rebuild the vault, with the recovery timelock given or any common one"
    )]
    RecoveryKeyMismatch,

    #[error(
        "The recovery key matches, but only {revealed} of the vault's custody members have signed a spend so far, so the rest of them aren't known yet"
    )]
    IncompleteCustody { revealed: usize },

    #[error("The search for the vault's recovery timelock stopped unexpectedly")]
    SearchFailed,

    #[error("Wallet error: {0}")]
    Wallet(#[from] WalletError),
}

/// The custody side of a vault, as revealed by spends through it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustodyReveal {
    /// The vault's custody hash, which is its singleton's inner puzzle hash.
    pub custody_hash: Bytes32,
    /// The hash of the left side of the top level 1 of 2.
    pub custody_member_hash: TreeHash,
    /// The hash of the top level 1 of 2, inside its index wrapper and delegated puzzle feeder.
    pub top_level_hash: TreeHash,
    /// How many members must sign, or `None` for a single member that isn't in an M of N.
    pub threshold: Option<usize>,
    /// The members which have been revealed, which are all of them unless it's an M of N.
    pub members: Vec<VaultMember>,
}

impl CustodyReveal {
    /// Adds the members revealed by another spend through the same custody.
    pub fn merge(&mut self, other: &Self) {
        for member in &other.members {
            if !self.members.contains(member) {
                self.members.push(*member);
            }
        }
    }

    /// The custody signers, if every member has been revealed.
    pub fn signers(&self) -> Option<VaultSigners> {
        let signers = VaultSigners::new(self.members.clone(), self.threshold.unwrap_or(1));
        (signers_hash(&signers, vec![]).ok()? == self.custody_member_hash).then_some(signers)
    }
}

/// The recovery side of a vault, found by searching for the recovery timelock and version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecoveryMatch {
    pub member: VaultMember,
    pub recovery_timelock: u64,
    pub version: VaultVersion,
}

/// The members that a recovery key could be, in either of Cloud Wallet's members for its curve.
pub fn recovery_key_members(key: RecoveryKey) -> Vec<VaultMember> {
    match key {
        RecoveryKey::Bls(public_key) => [false, true]
            .map(|fast_forward| VaultMember::Bls {
                public_key,
                fast_forward,
            })
            .to_vec(),
        RecoveryKey::Secp(bytes) => {
            let mut members = Vec::new();

            for fast_forward in [true, false] {
                if let Ok(public_key) = R1PublicKey::from_bytes(&bytes) {
                    members.push(VaultMember::Secp256r1 {
                        public_key,
                        fast_forward,
                    });
                    members.push(VaultMember::Passkey {
                        public_key,
                        fast_forward,
                    });
                }

                if let Ok(public_key) = K1PublicKey::from_bytes(&bytes) {
                    members.push(VaultMember::Secp256k1 {
                        public_key,
                        fast_forward,
                    });
                }
            }

            members
        }
    }
}

/// A vault's recovery key, which is a single member of its recovery side.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryKey {
    Bls(PublicKey),
    /// A compressed secp256r1 or secp256k1 public key, which could be either curve.
    Secp([u8; 33]),
}

/// The recovery timelocks that are tried when it isn't known: Cloud Wallet's default, then every
/// minute up to a week, every hour up to a year, and every day up to ten years. Cloud Wallet sets
/// timelocks in days, hours, and minutes.
pub fn common_recovery_timelocks() -> Vec<u64> {
    let mut timelocks = vec![DEFAULT_RECOVERY_TIMELOCK];

    timelocks.extend((1..=7 * 24 * 60).map(|minutes| minutes * MINUTE));
    timelocks.extend((1..=365 * 24).map(|hours| hours * HOUR));
    timelocks.extend((1..=10 * 365).map(|days| days * DAY));

    let mut seen = HashSet::new();
    timelocks.retain(|timelock| seen.insert(*timelock));
    timelocks
}

/// Searches for the recovery side which, with the revealed custody side, rebuilds the custody hash.
pub fn find_recovery(
    reveal: &CustodyReveal,
    recovery_members: &[VaultMember],
    timelocks: &[u64],
) -> Option<RecoveryMatch> {
    let member_hashes: Vec<(VaultMember, TreeHash)> = recovery_members
        .iter()
        .map(|&member| (member, member_puzzle_hash(member)))
        .collect();

    for version in [VaultVersion::ForceSingletonRecreation, VaultVersion::Legacy] {
        let hasher = RecoveryHasher::new(reveal.custody_member_hash, version);

        let member_tails: Vec<(VaultMember, TreeHash)> = member_hashes
            .iter()
            .map(|&(member, member_hash)| (member, hasher.member_tail(member_hash)))
            .collect();

        for &recovery_timelock in timelocks {
            let validator = hasher.validator(recovery_timelock);

            for &(member, member_tail) in &member_tails {
                let recovery_member_hash = hasher.member_hash(validator, member_tail);

                if MofN::new(1, vec![reveal.custody_member_hash, recovery_member_hash])
                    .inner_puzzle_hash()
                    == reveal.top_level_hash
                {
                    return Some(RecoveryMatch {
                        member,
                        recovery_timelock,
                        version,
                    });
                }
            }
        }
    }

    None
}

/// Hashes the recovery side of a vault for many timelocks, as [`recovery_restrictions`] and
/// [`wrap_mips_puzzle_hash`](chia_wallet_sdk::driver::wrap_mips_puzzle_hash) do, but with
/// everything that doesn't depend on the timelock hashed once. Searching every common timelock
/// hashes the recovery side tens of thousands of times.
struct RecoveryHasher {
    curry: CurryHasher,
    /// The curried arguments of the force 1 of 2 restriction before the member validator list,
    /// which holds the timelock.
    force_prefix: [TreeHash; 6],
    /// The quoted arguments of the force 1 of 2 restriction after the member validator list.
    force_tail: TreeHash,
    quoted_add_wrapper: TreeHash,
    /// The quoted wrapper stack after the force 1 of 2 restriction, which comes first.
    wrapper_stack_rest: TreeHash,
    restrictions_empty_list: TreeHash,
    index_nonce: TreeHash,
}

impl RecoveryHasher {
    fn new(custody_member_hash: TreeHash, version: VaultVersion) -> Self {
        let curry = CurryHasher::new();

        let force = Force1of2RestrictedVariable::new(
            custody_member_hash.into(),
            0,
            Bytes32::default(),
            ().tree_hash().into(),
        );

        let force_prefix = [
            force.delegated_puzzle_feeder_mod_hash.tree_hash(),
            force.one_of_n_mod_hash.tree_hash(),
            force.left_side_subtree_hash_hash.tree_hash(),
            force.index_wrapper_mod_hash.tree_hash(),
            force.nonce.tree_hash(),
            force.restriction_mod_hash.tree_hash(),
        ];
        let force_tail = curry.quote_args(
            &[force.delegated_puzzle_validator_list_hash.tree_hash()],
            curry.env,
        );

        let rest: Vec<TreeHash> = recovery_restrictions(custody_member_hash, 0, version)
            .into_iter()
            .skip(1)
            .map(|restriction| restriction.puzzle_hash)
            .collect();
        let enforce = EnforceDelegatedPuzzleWrappers::new(&rest);

        Self {
            curry,
            force_prefix,
            force_tail,
            quoted_add_wrapper: enforce.quoted_add_wrapper_mod_hash.tree_hash(),
            wrapper_stack_rest: enforce.quoted_wrapper_stack.tree_hash(),
            restrictions_empty_list: Vec::<TreeHash>::new().tree_hash(),
            index_nonce: 0_usize.tree_hash(),
        }
    }

    /// The hash of the delegated puzzle validator which enforces the recovery side's wrappers.
    fn validator(&self, recovery_timelock: u64) -> TreeHash {
        let timelock = self.curry.curry(
            Timelock::mod_hash(),
            &[recovery_timelock.tree_hash()],
            self.curry.env,
        );
        let member_validators = Bytes32::from(tree_hash_pair(timelock, self.curry.nil));

        let force_args = self
            .curry
            .quote_args(&[member_validators.tree_hash()], self.force_tail);
        let force = self.curry.curry(
            Force1of2RestrictedVariable::mod_hash(),
            &self.force_prefix,
            force_args,
        );

        let quoted_force = Bytes32::from(tree_hash_pair(self.curry.q, force));
        let wrapper_stack = tree_hash_pair(quoted_force.tree_hash(), self.wrapper_stack_rest);

        self.curry.curry(
            EnforceDelegatedPuzzleWrappers::mod_hash(),
            &[self.quoted_add_wrapper, wrapper_stack],
            self.curry.env,
        )
    }

    /// The quoted arguments of a member's restrictions after the delegated puzzle validators,
    /// which is just the member.
    fn member_tail(&self, member_hash: TreeHash) -> TreeHash {
        self.curry.quote_args(&[member_hash], self.curry.env)
    }

    /// The hash of the recovery side, with the member's restrictions enforced by the validator.
    fn member_hash(&self, validator: TreeHash, member_tail: TreeHash) -> TreeHash {
        let validators = tree_hash_pair(validator, self.curry.nil);
        let restrictions = self.curry.curry(
            RESTRICTIONS_HASH.into(),
            &[self.restrictions_empty_list, validators],
            member_tail,
        );

        self.curry.curry(
            INDEX_WRAPPER_HASH,
            &[self.index_nonce, restrictions],
            self.curry.env,
        )
    }
}

/// Curried puzzle hashes, as in [`curry_tree_hash`](chia_wallet_sdk::clvm_utils::curry_tree_hash),
/// but with the operators hashed once, and arguments that are fixed hashed ahead of time.
struct CurryHasher {
    nil: TreeHash,
    q: TreeHash,
    a: TreeHash,
    c: TreeHash,
    /// The environment that curried arguments are consed onto.
    env: TreeHash,
}

impl CurryHasher {
    fn new() -> Self {
        Self {
            nil: tree_hash_atom(&[]),
            q: tree_hash_atom(&[1]),
            a: tree_hash_atom(&[2]),
            c: tree_hash_atom(&[4]),
            env: tree_hash_atom(&[1]),
        }
    }

    /// Conses the quoted arguments onto the quoted arguments that come after them.
    fn quote_args(&self, args: &[TreeHash], tail: TreeHash) -> TreeHash {
        args.iter().rev().fold(tail, |quoted_args, &arg| {
            let quoted_arg = tree_hash_pair(self.q, arg);
            let rest = tree_hash_pair(quoted_args, self.nil);
            tree_hash_pair(self.c, tree_hash_pair(quoted_arg, rest))
        })
    }

    /// The hash of a puzzle curried with the arguments, followed by the quoted arguments in `tail`.
    fn curry(&self, mod_hash: TreeHash, args: &[TreeHash], tail: TreeHash) -> TreeHash {
        let quoted_args = self.quote_args(args, tail);
        let quoted_mod = tree_hash_pair(self.q, mod_hash);
        let rest = tree_hash_pair(quoted_args, self.nil);
        tree_hash_pair(self.a, tree_hash_pair(quoted_mod, rest))
    }
}

/// Rebuilds the config of every custody that a vault's spends reveal, from the latest spend back,
/// until its current custody is rebuilt.
///
/// The configs are verified against their custody hashes, so they can be trusted. The latest is
/// first.
pub async fn rebuild_vault_configs(
    peer: &WalletPeer,
    launcher_id: Bytes32,
    genesis_challenge: Bytes32,
    recovery_members: Vec<VaultMember>,
    timelocks: Vec<u64>,
) -> Result<Vec<VaultConfigRow>, VaultHistoryError> {
    let lineage = fetch_vault_lineage(peer, launcher_id, genesis_challenge)
        .await?
        .ok_or(VaultHistoryError::NotFound)?;

    let latest = *lineage.last().ok_or(VaultHistoryError::NotFound)?;
    let is_current = |custody_hash: Bytes32| {
        SingletonArgs::curry_tree_hash(launcher_id, custody_hash.into())
            == latest.coin.puzzle_hash.into()
    };

    let recovery_members = Arc::new(recovery_members);
    let timelocks = Arc::new(timelocks);

    let mut reveals: HashMap<Bytes32, CustodyReveal> = HashMap::new();
    let mut recoveries: HashMap<Bytes32, Option<RecoveryMatch>> = HashMap::new();
    let mut resolved = HashSet::new();
    let mut configs = Vec::new();

    let spent = lineage
        .iter()
        .rev()
        .filter_map(|coin_state| Some((coin_state.coin.coin_id(), coin_state.spent_height?)))
        .take(MAX_VAULT_SPENDS_CHECKED);

    for (coin_id, spent_height) in spent {
        let (puzzle, solution) = peer.fetch_puzzle_solution(coin_id, spent_height).await?;

        let Some(reveal) = parse_vault_spend(launcher_id, &puzzle, &solution) else {
            continue;
        };

        let custody_hash = reveal.custody_hash;

        if resolved.contains(&custody_hash) {
            continue;
        }

        let reveal = if let Some(existing) = reveals.get_mut(&custody_hash) {
            existing.merge(&reveal);
            existing.clone()
        } else {
            reveals.insert(custody_hash, reveal.clone());
            reveal
        };

        // The recovery side is the same for every spend with the same custody hash, so it's only
        // searched for once.
        let recovery = if let Some(recovery) = recoveries.get(&custody_hash) {
            *recovery
        } else {
            let search = reveal.clone();
            let recovery_members = recovery_members.clone();
            let timelocks = timelocks.clone();

            let recovery = tokio::task::spawn_blocking(move || {
                find_recovery(&search, &recovery_members, &timelocks)
            })
            .await
            .map_err(|_| VaultHistoryError::SearchFailed)?;

            recoveries.insert(custody_hash, recovery);
            recovery
        };

        let (Some(recovery), Some(custody)) = (recovery, reveal.signers()) else {
            continue;
        };

        let config = VaultConfig {
            custody,
            recovery: VaultSigners::new(vec![recovery.member], 1),
            recovery_timelock: recovery.recovery_timelock,
            version: recovery.version,
        };

        // This holds by construction, but the config is stored as trusted.
        if vault_hashes(&config)
            .is_ok_and(|hashes| hashes.custody_hash == TreeHash::from(custody_hash))
        {
            resolved.insert(custody_hash);
            configs.push(VaultConfigRow {
                custody_hash,
                layout: VaultLayout::Custody(config),
                source: VaultConfigSource::Reconstructed,
            });

            if is_current(custody_hash) {
                break;
            }
        }
    }

    if !configs.is_empty() {
        return Ok(configs);
    }

    if let Some(reveal) = reveals.values().find(|reveal| {
        recoveries
            .get(&reveal.custody_hash)
            .copied()
            .flatten()
            .is_some()
    }) {
        return Err(VaultHistoryError::IncompleteCustody {
            revealed: reveal.members.len(),
        });
    }

    if reveals.is_empty() {
        return Err(VaultHistoryError::NoCustodySpends);
    }

    Err(VaultHistoryError::RecoveryKeyMismatch)
}

/// Parses the custody side revealed by a spend of a vault singleton coin. Returns `None` if the
/// coin was spent through its recovery side, or isn't in Cloud Wallet's layout.
pub fn parse_vault_spend(
    launcher_id: Bytes32,
    puzzle: &Program,
    solution: &Program,
) -> Option<CustodyReveal> {
    let mut allocator = Allocator::new();
    let puzzle = puzzle.to_clvm(&mut allocator).ok()?;
    let solution = solution.to_clvm(&mut allocator).ok()?;

    let singleton =
        SingletonLayer::<Puzzle>::parse_puzzle(&allocator, Puzzle::parse(&allocator, puzzle))
            .ok()??;

    if singleton.launcher_id != launcher_id {
        return None;
    }

    let solution = SingletonLayer::<Puzzle>::parse_solution(&allocator, solution).ok()?;

    parse_custody_reveal(
        &allocator,
        singleton.inner_puzzle.ptr(),
        solution.inner_solution,
    )
}

/// Parses the custody side revealed by a spend of a vault's inner puzzle, which is a top level 1 of
/// 2 of `[custody, recovery]`.
pub fn parse_custody_reveal(
    allocator: &Allocator,
    inner_puzzle: NodePtr,
    inner_solution: NodePtr,
) -> Option<CustodyReveal> {
    let feeder = index_wrapper_inner(allocator, inner_puzzle)?;
    let feeder = curried_args(allocator, feeder, DELEGATED_PUZZLE_FEEDER_HASH.into())?;
    let feeder = DelegatedPuzzleFeederArgs::<NodePtr>::from_clvm(allocator, feeder).ok()?;

    curried_args(allocator, feeder.inner_puzzle, OneOfNArgs::mod_hash())?;
    let top_level_hash = tree_hash(allocator, feeder.inner_puzzle);

    // The index wrapper passes its solution through to the delegated puzzle feeder.
    let solution = DelegatedPuzzleFeederSolution::<NodePtr, NodePtr, NodePtr>::from_clvm(
        allocator,
        inner_solution,
    )
    .ok()?;
    let solution =
        OneOfNSolution::<NodePtr, NodePtr>::from_clvm(allocator, solution.inner_solution).ok()?;

    // The recovery side is wrapped in restrictions, so it isn't parsed as signers.
    let side = index_wrapper_inner(allocator, solution.member_puzzle)?;
    let (threshold, members) = parse_signers(allocator, side, solution.member_solution)?;

    Some(CustodyReveal {
        custody_hash: tree_hash(allocator, inner_puzzle).into(),
        custody_member_hash: tree_hash(allocator, solution.member_puzzle),
        top_level_hash,
        threshold,
        members,
    })
}

/// Parses a single member, or an M of N of members and the ones its solution reveals.
fn parse_signers(
    allocator: &Allocator,
    puzzle: NodePtr,
    solution: NodePtr,
) -> Option<(Option<usize>, Vec<VaultMember>)> {
    if let Some(member) = parse_member(allocator, puzzle) {
        return Some((None, vec![member]));
    }

    let curried = Puzzle::parse(allocator, puzzle).as_curried()?;

    if curried.mod_hash == OneOfNArgs::mod_hash() {
        let solution = OneOfNSolution::<NodePtr, NodePtr>::from_clvm(allocator, solution).ok()?;
        let member = index_wrapper_inner(allocator, solution.member_puzzle)?;
        return Some((Some(1), vec![parse_member(allocator, member)?]));
    }

    if curried.mod_hash == NofNArgs::<NodePtr>::mod_hash() {
        let args = NofNArgs::<NodePtr>::from_clvm(allocator, curried.args).ok()?;

        let members = args
            .members
            .iter()
            .map(|&member| parse_member(allocator, index_wrapper_inner(allocator, member)?))
            .collect::<Option<Vec<_>>>()?;

        return Some((Some(members.len()), members));
    }

    if curried.mod_hash == MofNArgs::mod_hash() {
        let args = MofNArgs::from_clvm(allocator, curried.args).ok()?;
        let solution = MofNSolution::<NodePtr>::from_clvm(allocator, solution).ok()?;

        let mut members = Vec::new();
        revealed_leaves(allocator, solution.proofs, &mut members, 0)?;

        return Some((Some(args.required), members));
    }

    None
}

/// Collects the members revealed by an M of N proof. Revealed leaves are `(() puzzle . solution)`,
/// and hidden subtrees are their hashes.
fn revealed_leaves(
    allocator: &Allocator,
    proof: NodePtr,
    members: &mut Vec<VaultMember>,
    depth: usize,
) -> Option<()> {
    if depth > MAX_PROOF_DEPTH {
        return None;
    }

    let SExp::Pair(first, rest) = allocator.sexp(proof) else {
        return Some(());
    };

    if first == NodePtr::NIL {
        let (puzzle, _solution) = <(NodePtr, NodePtr)>::from_clvm(allocator, rest).ok()?;
        let member = parse_member(allocator, index_wrapper_inner(allocator, puzzle)?)?;

        if !members.contains(&member) {
            members.push(member);
        }

        return Some(());
    }

    revealed_leaves(allocator, first, members, depth + 1)?;
    revealed_leaves(allocator, rest, members, depth + 1)
}

/// Parses a member puzzle in Cloud Wallet's layout.
fn parse_member(allocator: &Allocator, puzzle: NodePtr) -> Option<VaultMember> {
    let curried = Puzzle::parse(allocator, puzzle).as_curried()?;
    let args = curried.args;
    let mod_hash = curried.mod_hash;

    let member = if mod_hash == BlsMember::mod_hash() {
        VaultMember::Bls {
            public_key: BlsMember::from_clvm(allocator, args).ok()?.public_key,
            fast_forward: false,
        }
    } else if mod_hash == BlsMemberPuzzleAssert::mod_hash() {
        VaultMember::Bls {
            public_key: BlsMemberPuzzleAssert::from_clvm(allocator, args)
                .ok()?
                .public_key,
            fast_forward: true,
        }
    } else if mod_hash == R1Member::mod_hash() {
        VaultMember::Secp256r1 {
            public_key: R1Member::from_clvm(allocator, args).ok()?.public_key,
            fast_forward: false,
        }
    } else if mod_hash == R1MemberPuzzleAssert::mod_hash() {
        VaultMember::Secp256r1 {
            public_key: R1MemberPuzzleAssert::from_clvm(allocator, args)
                .ok()?
                .public_key,
            fast_forward: true,
        }
    } else if mod_hash == K1Member::mod_hash() {
        VaultMember::Secp256k1 {
            public_key: K1Member::from_clvm(allocator, args).ok()?.public_key,
            fast_forward: false,
        }
    } else if mod_hash == K1MemberPuzzleAssert::mod_hash() {
        VaultMember::Secp256k1 {
            public_key: K1MemberPuzzleAssert::from_clvm(allocator, args)
                .ok()?
                .public_key,
            fast_forward: true,
        }
    } else if mod_hash == PasskeyMember::mod_hash() {
        VaultMember::Passkey {
            public_key: PasskeyMember::from_clvm(allocator, args).ok()?.public_key,
            fast_forward: false,
        }
    } else if mod_hash == PasskeyMemberPuzzleAssert::mod_hash() {
        VaultMember::Passkey {
            public_key: PasskeyMemberPuzzleAssert::from_clvm(allocator, args)
                .ok()?
                .public_key,
            fast_forward: true,
        }
    } else if mod_hash == SingletonMember::mod_hash() {
        VaultMember::Singleton {
            launcher_id: SingletonMember::from_clvm(allocator, args)
                .ok()?
                .singleton_struct
                .launcher_id,
            fast_forward: false,
        }
    } else if mod_hash == SingletonMemberWithMode::mod_hash() {
        let member = SingletonMemberWithMode::from_clvm(allocator, args).ok()?;

        if member.mode != FAST_FORWARD_SINGLETON_MODE {
            return None;
        }

        VaultMember::Singleton {
            launcher_id: member.singleton_struct.launcher_id,
            fast_forward: true,
        }
    } else {
        return None;
    };

    // The member must be exactly the puzzle, rather than one with other curried values, such as a
    // singleton with a different singleton mod.
    (member_puzzle_hash(member) == tree_hash(allocator, puzzle)).then_some(member)
}

/// The inner puzzle of an index wrapper with a nonce of 0, which every puzzle in Cloud Wallet's
/// layout has.
fn index_wrapper_inner(allocator: &Allocator, puzzle: NodePtr) -> Option<NodePtr> {
    let args = curried_args(allocator, puzzle, INDEX_WRAPPER_HASH)?;
    let args = IndexWrapperArgs::<usize, NodePtr>::from_clvm(allocator, args).ok()?;
    (args.nonce == 0).then_some(args.inner_puzzle)
}

fn curried_args(allocator: &Allocator, puzzle: NodePtr, mod_hash: TreeHash) -> Option<NodePtr> {
    let curried = Puzzle::parse(allocator, puzzle).as_curried()?;
    (curried.mod_hash == mod_hash).then_some(curried.args)
}
