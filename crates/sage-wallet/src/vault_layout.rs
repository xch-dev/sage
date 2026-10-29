//! Chia Cloud Wallet's vault layout, as built by `getVaultInternals` and
//! `getVaultMipsMemoListBytes` in ent-wallet, and decoded by `decodeVaultMemoList`.
//!
//! The vault's inner puzzle is a top level 1 of 2 of `[custody, recovery]`:
//!
//! - Custody is a single member, or an M of N of members sorted by their hashes.
//! - Recovery is a single member, or an M of N of members sorted by their hashes, which is
//!   restricted so that it can only start recovery. It's forced to replace itself with the
//!   timelocked recovery finish member, can't create announcements or send or receive messages,
//!   can only create one coin, and in [`VaultVersion::ForceSingletonRecreation`] vaults must
//!   recreate the singleton.
//! - Once recovery has started, recovery is replaced by a timelocked member which recreates the
//!   vault with its new custody, and the CHIP-0043 memo of the new custody.
//!
//! Every nonce is 0. The CHIP-0043 memo list is `[memo]`, which reveals every key.

#[cfg(test)]
mod tests;

use chia_wallet_sdk::{
    chia::{
        consensus::opcodes::{
            CREATE_COIN_ANNOUNCEMENT, CREATE_PUZZLE_ANNOUNCEMENT, RECEIVE_MESSAGE, SEND_MESSAGE,
        },
        puzzle_types::Memos,
    },
    clvm_traits::{FromClvm, ToClvm, clvm_quote},
    clvm_utils::{ToTreeHash, TreeHash, tree_hash},
    clvmr::{Allocator, NodePtr, error::EvalErr, serde::node_from_bytes},
    driver::{
        InnerPuzzleMemo, MemberMemo, MemoKind, MipsMemo, MipsMemoContext, MofN, MofNMemo,
        P2Singleton, ParsedMember, ParsedRestriction, ParsedWrapper, Restriction, RestrictionKind,
        RestrictionMemo, VaultVersion, WrapperMemo, mips_puzzle_hash,
    },
    prelude::*,
    puzzles::PREVENT_MULTIPLE_CREATE_COINS_HASH,
    types::{
        Condition, Mod,
        puzzles::{
            BlsMember, BlsMemberPuzzleAssert, FORCE_SINGLETON_RECREATION_HASH,
            Force1of2RestrictedVariable, K1Member, K1MemberPuzzleAssert, PasskeyMember,
            PasskeyMemberPuzzleAssert, PreventConditionOpcode, R1Member, R1MemberPuzzleAssert,
            SingletonMember, SingletonMemberWithMode, Timelock,
        },
    },
};
use sage_database::{VaultConfig, VaultMember, VaultRecoveryConfig, VaultSigners};
use thiserror::Error;

/// Cloud Wallet's memo lists are around a kilobyte, so this leaves plenty of room for large
/// M of N vaults, without decoding arbitrarily large untrusted input.
pub const MAX_VAULT_MEMO_LIST_SIZE: usize = 65_536;

/// The mode of fast forwardable singleton members, as in [`MemberMemo::singleton`].
const FAST_FORWARD_SINGLETON_MODE: u8 = 0b010_010;

/// The condition opcodes that the recovery path is prevented from using, in order.
const PREVENTED_RECOVERY_OPCODES: [u16; 4] = [
    CREATE_COIN_ANNOUNCEMENT,
    CREATE_PUZZLE_ANNOUNCEMENT,
    SEND_MESSAGE,
    RECEIVE_MESSAGE,
];

#[derive(Debug, Error)]
pub enum VaultLayoutError {
    #[error(
        "Vault signers must have at least one member, and a threshold between 1 and the number of members"
    )]
    InvalidSigners,

    #[error(
        "Custom members can't be revealed in a vault memo, so they aren't part of Cloud Wallet's vault layout"
    )]
    CustomMember,

    #[error("The vault memo list is larger than {MAX_VAULT_MEMO_LIST_SIZE} bytes")]
    MemoTooLarge,

    #[error("The vault memo doesn't match Cloud Wallet's vault layout")]
    UnsupportedMemo,

    #[error("The vault memo doesn't rebuild the vault's custody hash")]
    HashMismatch,

    #[error("The post-recovery custody doesn't rebuild the recovery custody hash")]
    RecoveryCustodyHashMismatch,

    #[error("Driver error: {0}")]
    Driver(#[from] DriverError),

    #[error("CLVM encoding error: {0}")]
    ToClvm(#[from] chia_wallet_sdk::clvm_traits::ToClvmError),

    #[error("CLVM decoding error: {0}")]
    FromClvm(#[from] chia_wallet_sdk::clvm_traits::FromClvmError),

    #[error("CLVM error: {0}")]
    Clvm(#[from] EvalErr),
}

/// The hashes of a vault's inner puzzle and its two sides.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VaultHashes {
    /// The vault singleton's inner puzzle hash, which is the custody hash of [`VaultInfo`].
    pub custody_hash: TreeHash,
    /// The hash of the left side of the top level 1 of 2, which can spend the vault.
    pub custody_member_hash: TreeHash,
    /// The hash of the right side of the top level 1 of 2, which starts or finishes recovery.
    pub recovery_member_hash: TreeHash,
}

#[derive(Debug, Clone)]
pub struct BuiltVault {
    pub hashes: VaultHashes,
    pub memo: MipsMemo,
    /// The CHIP-0043 memo list `[memo]`, which is attached to the vault's singleton child.
    pub memo_list: NodePtr,
    /// The quoted conditions of the recovery finish member, if the vault is in the recovery state.
    pub recovery_finish_puzzle: Option<NodePtr>,
}

/// A vault decoded from a CHIP-0043 memo list, which was verified to rebuild its custody hash.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodedVault {
    Custody(VaultConfig),
    Recovery {
        config: VaultRecoveryConfig,
        /// The vault's config once recovery finishes, which was revealed by the recovery finish
        /// member, and rebuilds [`VaultRecoveryConfig::recovery_custody_hash`].
        post_recovery: VaultConfig,
    },
}

/// The puzzle hash of a vault's P2 singleton address with the given nonce. Coins sent to it can
/// be spent by the vault, regardless of its custody.
pub fn p2_singleton_puzzle_hash(launcher_id: Bytes32, nonce: usize) -> Bytes32 {
    P2Singleton::new(launcher_id, nonce).tree_hash().into()
}

/// Calculates the hashes of a vault in its normal state, without allocating its memo.
pub fn vault_hashes(config: &VaultConfig) -> Result<VaultHashes, VaultLayoutError> {
    let custody_member_hash = signers_hash(&config.custody, vec![])?;
    let recovery_member_hash = signers_hash(
        &config.recovery,
        recovery_restrictions(
            custody_member_hash,
            config.recovery_timelock,
            config.version,
        ),
    )?;

    Ok(top_level_hashes(custody_member_hash, recovery_member_hash))
}

/// Builds a vault in its normal state, as in `getVaultInternals` and `getVaultMipsMemoListBytes`.
pub fn build_vault(
    allocator: &mut Allocator,
    config: &VaultConfig,
) -> Result<BuiltVault, VaultLayoutError> {
    let hashes = vault_hashes(config)?;

    let custody = signers_memo(allocator, &config.custody, vec![])?;

    let timelock = RestrictionMemo::timelock(allocator, config.recovery_timelock, true)?;

    let mut wrappers = vec![WrapperMemo::force_1_of_2_restricted_variable(
        allocator,
        custody.inner_puzzle_hash(false).into(),
        0,
        vec![timelock],
    )?];

    for opcode in PREVENTED_RECOVERY_OPCODES {
        wrappers.push(WrapperMemo::prevent_condition_opcode(
            allocator, opcode, true,
        )?);
    }

    wrappers.push(WrapperMemo::prevent_multiple_create_coins());

    if config.version == VaultVersion::ForceSingletonRecreation {
        wrappers.push(WrapperMemo::force_singleton_recreation());
    }

    let restriction = RestrictionMemo::enforce_delegated_puzzle_wrappers(allocator, &wrappers)?;
    let recovery = signers_memo(allocator, &config.recovery, vec![restriction])?;

    finish_build(allocator, hashes, custody, recovery, None)
}

/// Builds a vault whose recovery has started. The post-recovery config must rebuild
/// [`VaultRecoveryConfig::recovery_custody_hash`], since its memo is revealed when recovery
/// finishes.
pub fn build_vault_recovery(
    allocator: &mut Allocator,
    config: &VaultRecoveryConfig,
    post_recovery: &VaultConfig,
) -> Result<BuiltVault, VaultLayoutError> {
    let post_recovery = build_vault(allocator, post_recovery)?;

    if post_recovery.hashes.custody_hash != config.recovery_custody_hash.into() {
        return Err(VaultLayoutError::RecoveryCustodyHashMismatch);
    }

    let finish_puzzle = recovery_finish_puzzle(allocator, config, post_recovery.memo_list)?;
    let finish_puzzle_hash = tree_hash(allocator, finish_puzzle);

    let custody_member_hash = signers_hash(&config.custody, vec![])?;
    let recovery_member_hash = mips_puzzle_hash(
        0,
        vec![Restriction {
            kind: RestrictionKind::MemberCondition,
            puzzle_hash: Timelock::new(config.recovery_timelock).curry_tree_hash(),
        }],
        finish_puzzle_hash,
        false,
    );
    let hashes = top_level_hashes(custody_member_hash, recovery_member_hash);

    let custody = signers_memo(allocator, &config.custody, vec![])?;
    let recovery = InnerPuzzleMemo::new(
        0,
        vec![RestrictionMemo::timelock(
            allocator,
            config.recovery_timelock,
            true,
        )?],
        MemoKind::Member(MemberMemo::new(finish_puzzle_hash.into(), finish_puzzle)),
    );

    finish_build(allocator, hashes, custody, recovery, Some(finish_puzzle))
}

/// Decodes and verifies the CHIP-0043 memo list attached to a vault singleton with the given
/// custody hash, as in `decodeVaultMemoList`.
///
/// Memos are untrusted, so this only accepts a memo if rebuilding the vault from it reproduces the
/// custody hash. Every member must reveal a key or launcher id, as Cloud Wallet's memos do.
pub fn decode_vault_memo_list(
    allocator: &mut Allocator,
    memo_list: NodePtr,
    custody_hash: Bytes32,
) -> Result<DecodedVault, VaultLayoutError> {
    decode_memo_list(allocator, memo_list, custody_hash, true)
}

/// Like [`decode_vault_memo_list`], but for a serialized memo list, which can't be larger than
/// [`MAX_VAULT_MEMO_LIST_SIZE`].
pub fn decode_vault_memo_list_bytes(
    allocator: &mut Allocator,
    memo_list: &[u8],
    custody_hash: Bytes32,
) -> Result<DecodedVault, VaultLayoutError> {
    if memo_list.len() > MAX_VAULT_MEMO_LIST_SIZE {
        return Err(VaultLayoutError::MemoTooLarge);
    }

    let memo_list = node_from_bytes(allocator, memo_list)?;

    decode_vault_memo_list(allocator, memo_list, custody_hash)
}

/// The custody hash that a CHIP-0043 memo list claims to rebuild, without decoding its layout.
///
/// This is only a candidate, which must match a puzzle hash before the memo can be trusted. It's
/// used to find the custody hash of an eve vault coin from its launcher's memo, since the eve coin
/// doesn't reveal its inner puzzle until it's spent.
pub fn vault_memo_list_custody_hash(allocator: &Allocator, memo_list: NodePtr) -> Option<Bytes32> {
    let (memo, _rest) = <(NodePtr, NodePtr)>::from_clvm(allocator, memo_list).ok()?;
    let memo = MipsMemo::from_clvm_bounded(allocator, memo).ok()?;
    memo.try_inner_puzzle_hash().ok().map(Into::into)
}

fn top_level_hashes(custody_member_hash: TreeHash, recovery_member_hash: TreeHash) -> VaultHashes {
    VaultHashes {
        custody_hash: mips_puzzle_hash(
            0,
            vec![],
            MofN::new(1, vec![custody_member_hash, recovery_member_hash]).inner_puzzle_hash(),
            true,
        ),
        custody_member_hash,
        recovery_member_hash,
    }
}

fn finish_build(
    allocator: &mut Allocator,
    hashes: VaultHashes,
    custody: InnerPuzzleMemo,
    recovery: InnerPuzzleMemo,
    recovery_finish_puzzle: Option<NodePtr>,
) -> Result<BuiltVault, VaultLayoutError> {
    let memo = MipsMemo::new(InnerPuzzleMemo::new(
        0,
        vec![],
        MemoKind::MofN(MofNMemo::new(1, vec![custody, recovery])),
    ));

    // The memo and the hashes are built separately, so check that they agree, as Cloud Wallet does
    // before writing the memo.
    if !memo.verify(hashes.custody_hash) {
        return Err(VaultLayoutError::HashMismatch);
    }

    let memo_node = memo.to_clvm(allocator)?;
    let memo_list = vec![memo_node].to_clvm(allocator)?;

    Ok(BuiltVault {
        hashes,
        memo,
        memo_list,
        recovery_finish_puzzle,
    })
}

/// The restrictions on the recovery path before recovery has started, in order.
pub(crate) fn recovery_restrictions(
    custody_member_hash: TreeHash,
    recovery_timelock: u64,
    version: VaultVersion,
) -> Vec<Restriction> {
    let mut wrappers = vec![
        Force1of2RestrictedVariable::new(
            custody_member_hash.into(),
            0,
            vec![Timelock::new(recovery_timelock).curry_tree_hash()]
                .tree_hash()
                .into(),
            ().tree_hash().into(),
        )
        .curry_tree_hash(),
    ];

    wrappers.extend(
        PREVENTED_RECOVERY_OPCODES
            .into_iter()
            .map(|opcode| PreventConditionOpcode::new(opcode).curry_tree_hash()),
    );

    wrappers.push(PREVENT_MULTIPLE_CREATE_COINS_HASH.into());

    if version == VaultVersion::ForceSingletonRecreation {
        wrappers.push(FORCE_SINGLETON_RECREATION_HASH.into());
    }

    wrappers
        .into_iter()
        .map(|puzzle_hash| Restriction {
            kind: RestrictionKind::DelegatedPuzzleWrapper,
            puzzle_hash,
        })
        .collect()
}

/// The quoted conditions that finish recovery, by recreating the vault with its new custody.
fn recovery_finish_puzzle(
    allocator: &mut Allocator,
    config: &VaultRecoveryConfig,
    post_recovery_memo_list: NodePtr,
) -> Result<NodePtr, VaultLayoutError> {
    let conditions = match config.version {
        VaultVersion::Legacy => Conditions::new()
            .create_coin(
                config.recovery_custody_hash,
                1,
                Memos::Some(post_recovery_memo_list),
            )
            .assert_seconds_relative(config.recovery_timelock),
        VaultVersion::ForceSingletonRecreation => Conditions::new()
            .create_coin(
                config.recovery_custody_hash,
                config.amount,
                Memos::Some(post_recovery_memo_list),
            )
            .assert_seconds_relative(config.recovery_timelock)
            .assert_my_amount(config.amount),
    };

    Ok(clvm_quote!(conditions).to_clvm(allocator)?)
}

fn validate_signers(signers: &VaultSigners) -> Result<(), VaultLayoutError> {
    let count = signers.members.len();

    // A single member is used as is, so a threshold other than 1 would be ignored.
    if count == 0 || signers.threshold == 0 || signers.threshold > count {
        return Err(VaultLayoutError::InvalidSigners);
    }

    if count == 1 && signers.threshold != 1 {
        return Err(VaultLayoutError::InvalidSigners);
    }

    if signers
        .members
        .iter()
        .any(|member| matches!(member, VaultMember::Custom { .. }))
    {
        return Err(VaultLayoutError::CustomMember);
    }

    Ok(())
}

/// The curried puzzle hash of a member, as in `getMemberHash`.
fn member_puzzle_hash(member: VaultMember) -> TreeHash {
    match member {
        VaultMember::Bls {
            public_key,
            fast_forward: true,
        } => BlsMemberPuzzleAssert::new(public_key).curry_tree_hash(),
        VaultMember::Bls { public_key, .. } => BlsMember::new(public_key).curry_tree_hash(),
        VaultMember::Secp256r1 {
            public_key,
            fast_forward: true,
        } => R1MemberPuzzleAssert::new(public_key).curry_tree_hash(),
        VaultMember::Secp256r1 { public_key, .. } => R1Member::new(public_key).curry_tree_hash(),
        VaultMember::Secp256k1 {
            public_key,
            fast_forward: true,
        } => K1MemberPuzzleAssert::new(public_key).curry_tree_hash(),
        VaultMember::Secp256k1 { public_key, .. } => K1Member::new(public_key).curry_tree_hash(),
        VaultMember::Passkey {
            public_key,
            fast_forward: true,
        } => PasskeyMemberPuzzleAssert::new(public_key).curry_tree_hash(),
        VaultMember::Passkey { public_key, .. } => PasskeyMember::new(public_key).curry_tree_hash(),
        VaultMember::Singleton {
            launcher_id,
            fast_forward: true,
        } => {
            SingletonMemberWithMode::new(launcher_id, FAST_FORWARD_SINGLETON_MODE).curry_tree_hash()
        }
        VaultMember::Singleton { launcher_id, .. } => {
            SingletonMember::new(launcher_id).curry_tree_hash()
        }
        VaultMember::Custom { puzzle_hash } => puzzle_hash.into(),
    }
}

/// The hash of a signer tree, as in `getVaultInternals`.
fn signers_hash(
    signers: &VaultSigners,
    restrictions: Vec<Restriction>,
) -> Result<TreeHash, VaultLayoutError> {
    validate_signers(signers)?;

    if let [member] = signers.members.as_slice() {
        return Ok(mips_puzzle_hash(
            0,
            restrictions,
            member_puzzle_hash(*member),
            false,
        ));
    }

    let mut member_hashes: Vec<TreeHash> = signers
        .members
        .iter()
        .map(|&member| mips_puzzle_hash(0, vec![], member_puzzle_hash(member), false))
        .collect();

    member_hashes.sort();

    Ok(mips_puzzle_hash(
        0,
        restrictions,
        MofN::new(signers.threshold, member_hashes).inner_puzzle_hash(),
        false,
    ))
}

fn member_memo(
    allocator: &mut Allocator,
    member: VaultMember,
) -> Result<MemberMemo, VaultLayoutError> {
    Ok(match member {
        VaultMember::Bls {
            public_key,
            fast_forward,
        } => MemberMemo::bls(allocator, public_key, fast_forward, false, true)?,
        VaultMember::Secp256r1 {
            public_key,
            fast_forward,
        } => MemberMemo::r1(allocator, public_key, fast_forward, true)?,
        VaultMember::Secp256k1 {
            public_key,
            fast_forward,
        } => MemberMemo::k1(allocator, public_key, fast_forward, true)?,
        VaultMember::Passkey {
            public_key,
            fast_forward,
        } => MemberMemo::passkey(allocator, public_key, fast_forward, true)?,
        VaultMember::Singleton {
            launcher_id,
            fast_forward,
        } => MemberMemo::singleton(allocator, launcher_id, fast_forward, true)?,
        VaultMember::Custom { puzzle_hash } => MemberMemo::new(puzzle_hash, NodePtr::NIL),
    })
}

/// The memo of a signer tree, as in `createMipsSignerTree`.
fn signers_memo(
    allocator: &mut Allocator,
    signers: &VaultSigners,
    restrictions: Vec<RestrictionMemo>,
) -> Result<InnerPuzzleMemo, VaultLayoutError> {
    validate_signers(signers)?;

    let mut members = signers
        .members
        .iter()
        .map(|&member| {
            Ok(InnerPuzzleMemo::new(
                0,
                vec![],
                MemoKind::Member(member_memo(allocator, member)?),
            ))
        })
        .collect::<Result<Vec<_>, VaultLayoutError>>()?;

    if members.len() == 1 {
        let member = members.remove(0);
        return Ok(InnerPuzzleMemo::new(0, restrictions, member.kind));
    }

    members.sort_by_cached_key(|member| member.inner_puzzle_hash(false));

    Ok(InnerPuzzleMemo::new(
        0,
        restrictions,
        MemoKind::MofN(MofNMemo::new(signers.threshold, members)),
    ))
}

fn decode_memo_list(
    allocator: &mut Allocator,
    memo_list: NodePtr,
    custody_hash: Bytes32,
    allow_recovery_state: bool,
) -> Result<DecodedVault, VaultLayoutError> {
    // Cloud Wallet only reads the first memo in the list.
    let (memo, _rest) = <(NodePtr, NodePtr)>::from_clvm(allocator, memo_list)?;
    let memo = MipsMemo::from_clvm_bounded(allocator, memo)?;

    if !memo.verify(custody_hash.into()) {
        return Err(VaultLayoutError::HashMismatch);
    }

    let top_level = &memo.inner_puzzle;

    if top_level.nonce != 0 || !top_level.restrictions.is_empty() {
        return Err(VaultLayoutError::UnsupportedMemo);
    }

    let MemoKind::MofN(top_level) = &top_level.kind else {
        return Err(VaultLayoutError::UnsupportedMemo);
    };

    let (1, [custody, recovery]) = (top_level.required, top_level.items.as_slice()) else {
        return Err(VaultLayoutError::UnsupportedMemo);
    };

    if !custody.restrictions.is_empty() || recovery.nonce != 0 {
        return Err(VaultLayoutError::UnsupportedMemo);
    }

    let ctx = MipsMemoContext::default();
    let custody_signers = decode_signers(allocator, &ctx, custody)?;

    let [restriction] = recovery.restrictions.as_slice() else {
        return Err(VaultLayoutError::UnsupportedMemo);
    };

    let decoded = match restriction.parse(allocator, &ctx) {
        Some(ParsedRestriction::EnforceDelegatedPuzzleWrappers(_, wrappers)) => {
            let mut recovery_timelock = None;
            let mut version = VaultVersion::Legacy;

            // Everything else, including the left side of the force 1 of 2, is checked by
            // rebuilding the custody hash below.
            for wrapper in wrappers {
                match wrapper.parse(allocator, &ctx) {
                    Some(ParsedWrapper::Force1of2RestrictedVariable(_, restrictions)) => {
                        if recovery_timelock.is_some() {
                            return Err(VaultLayoutError::UnsupportedMemo);
                        }

                        let [restriction] = restrictions.as_slice() else {
                            return Err(VaultLayoutError::UnsupportedMemo);
                        };

                        let Some(ParsedRestriction::Timelock(timelock)) =
                            restriction.parse(allocator, &ctx)
                        else {
                            return Err(VaultLayoutError::UnsupportedMemo);
                        };

                        recovery_timelock = Some(timelock.seconds);
                    }
                    Some(ParsedWrapper::ForceSingletonRecreation) => {
                        version = VaultVersion::ForceSingletonRecreation;
                    }
                    _ => {}
                }
            }

            let Some(recovery_timelock) = recovery_timelock else {
                return Err(VaultLayoutError::UnsupportedMemo);
            };

            let config = VaultConfig {
                custody: custody_signers,
                recovery: decode_signers(allocator, &ctx, recovery)?,
                recovery_timelock,
                version,
            };

            if vault_hashes(&config)?.custody_hash != custody_hash.into() {
                return Err(VaultLayoutError::HashMismatch);
            }

            DecodedVault::Custody(config)
        }
        Some(ParsedRestriction::Timelock(timelock))
            if allow_recovery_state && restriction.member_condition_validator =>
        {
            let MemoKind::Member(finish) = &recovery.kind else {
                return Err(VaultLayoutError::UnsupportedMemo);
            };

            let Some(ParsedMember::Custom(finish_puzzle)) = finish.parse(allocator, &ctx) else {
                return Err(VaultLayoutError::UnsupportedMemo);
            };

            let finish = decode_recovery_finish_puzzle(allocator, finish_puzzle)?;

            // Cloud Wallet always finishes recovery with the same timelock that it waits for.
            if finish.timelock != timelock.seconds {
                return Err(VaultLayoutError::UnsupportedMemo);
            }

            let DecodedVault::Custody(post_recovery) = decode_memo_list(
                allocator,
                finish.memo_list,
                finish.recovery_custody_hash,
                false,
            )?
            else {
                return Err(VaultLayoutError::UnsupportedMemo);
            };

            let config = VaultRecoveryConfig {
                custody: custody_signers,
                recovery_timelock: timelock.seconds,
                version: finish.version,
                recovery_custody_hash: finish.recovery_custody_hash,
                amount: finish.amount,
            };

            let rebuilt = build_vault_recovery(allocator, &config, &post_recovery)?;

            if rebuilt.hashes.custody_hash != custody_hash.into() {
                return Err(VaultLayoutError::HashMismatch);
            }

            DecodedVault::Recovery {
                config,
                post_recovery,
            }
        }
        _ => return Err(VaultLayoutError::UnsupportedMemo),
    };

    Ok(decoded)
}

/// Decodes a signer tree, ignoring the restrictions on its root, as in `decodeSignerTree`.
fn decode_signers(
    allocator: &Allocator,
    ctx: &MipsMemoContext,
    memo: &InnerPuzzleMemo,
) -> Result<VaultSigners, VaultLayoutError> {
    if memo.nonce != 0 {
        return Err(VaultLayoutError::UnsupportedMemo);
    }

    match &memo.kind {
        MemoKind::Member(member) => Ok(VaultSigners::new(
            vec![decode_member(allocator, ctx, member)?],
            1,
        )),
        MemoKind::MofN(m_of_n) => {
            if m_of_n.items.len() < 2 {
                return Err(VaultLayoutError::UnsupportedMemo);
            }

            let members = m_of_n
                .items
                .iter()
                .map(|item| {
                    let MemoKind::Member(member) = &item.kind else {
                        return Err(VaultLayoutError::UnsupportedMemo);
                    };

                    if item.nonce != 0 || !item.restrictions.is_empty() {
                        return Err(VaultLayoutError::UnsupportedMemo);
                    }

                    decode_member(allocator, ctx, member)
                })
                .collect::<Result<_, _>>()?;

            Ok(VaultSigners::new(members, m_of_n.required))
        }
    }
}

fn decode_member(
    allocator: &Allocator,
    ctx: &MipsMemoContext,
    member: &MemberMemo,
) -> Result<VaultMember, VaultLayoutError> {
    Ok(match member.parse(allocator, ctx) {
        Some(ParsedMember::Bls(member)) => VaultMember::Bls {
            public_key: member.public_key,
            fast_forward: false,
        },
        Some(ParsedMember::BlsPuzzleAssert(member)) => VaultMember::Bls {
            public_key: member.public_key,
            fast_forward: true,
        },
        Some(ParsedMember::R1(member)) => VaultMember::Secp256r1 {
            public_key: member.public_key,
            fast_forward: false,
        },
        Some(ParsedMember::R1PuzzleAssert(member)) => VaultMember::Secp256r1 {
            public_key: member.public_key,
            fast_forward: true,
        },
        Some(ParsedMember::K1(member)) => VaultMember::Secp256k1 {
            public_key: member.public_key,
            fast_forward: false,
        },
        Some(ParsedMember::K1PuzzleAssert(member)) => VaultMember::Secp256k1 {
            public_key: member.public_key,
            fast_forward: true,
        },
        Some(ParsedMember::Passkey(member)) => VaultMember::Passkey {
            public_key: member.public_key,
            fast_forward: false,
        },
        Some(ParsedMember::PasskeyPuzzleAssert(member)) => VaultMember::Passkey {
            public_key: member.public_key,
            fast_forward: true,
        },
        Some(ParsedMember::Singleton(member)) => VaultMember::Singleton {
            launcher_id: member.singleton_struct.launcher_id,
            fast_forward: false,
        },
        Some(ParsedMember::SingletonWithMode(member))
            if member.mode == FAST_FORWARD_SINGLETON_MODE =>
        {
            VaultMember::Singleton {
                launcher_id: member.singleton_struct.launcher_id,
                fast_forward: true,
            }
        }
        _ => return Err(VaultLayoutError::UnsupportedMemo),
    })
}

struct RecoveryFinish {
    recovery_custody_hash: Bytes32,
    amount: u64,
    timelock: u64,
    version: VaultVersion,
    memo_list: NodePtr,
}

/// Reads the quoted conditions of the recovery finish member without running them, as in
/// `decodeRecoveryFinishPuzzle`.
fn decode_recovery_finish_puzzle(
    allocator: &Allocator,
    puzzle: NodePtr,
) -> Result<RecoveryFinish, VaultLayoutError> {
    let (quote, conditions) = <(u8, Vec<Condition<NodePtr>>)>::from_clvm(allocator, puzzle)?;

    if quote != 1 {
        return Err(VaultLayoutError::UnsupportedMemo);
    }

    let mut create_coin = None;
    let mut timelock = None;
    let mut asserted_amount = None;

    for condition in conditions {
        match condition {
            Condition::CreateCoin(condition) if create_coin.is_none() => {
                create_coin = Some(condition);
            }
            Condition::AssertSecondsRelative(condition) if timelock.is_none() => {
                timelock = Some(condition.seconds);
            }
            Condition::AssertMyAmount(condition) if asserted_amount.is_none() => {
                asserted_amount = Some(condition.amount);
            }
            _ => return Err(VaultLayoutError::UnsupportedMemo),
        }
    }

    let (Some(create_coin), Some(timelock)) = (create_coin, timelock) else {
        return Err(VaultLayoutError::UnsupportedMemo);
    };

    let Memos::Some(memo_list) = create_coin.memos else {
        return Err(VaultLayoutError::UnsupportedMemo);
    };

    Ok(RecoveryFinish {
        recovery_custody_hash: create_coin.puzzle_hash,
        amount: asserted_amount.unwrap_or(create_coin.amount),
        timelock,
        version: if asserted_amount.is_some() {
            VaultVersion::ForceSingletonRecreation
        } else {
            VaultVersion::Legacy
        },
        memo_list,
    })
}
