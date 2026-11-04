use chia_wallet_sdk::{
    driver::{InnerPuzzleSpend, MipsSpend, mips_puzzle_hash},
    test::{BlsPair, K1Pair, R1Pair},
    types::puzzles::{
        BlsMember, K1MemberPuzzleAssert, PasskeyMemberPuzzleAssert, R1MemberPuzzleAssert,
    },
};

use super::*;

fn bls_config(custody: VaultSigners, recovery: PublicKey, timelock: u64) -> VaultConfig {
    VaultConfig {
        custody,
        recovery: VaultSigners::new(vec![VaultMember::bls(recovery)], 1),
        recovery_timelock: timelock,
        version: VaultVersion::ForceSingletonRecreation,
    }
}

fn member_puzzle(ctx: &mut SpendContext, member: VaultMember) -> anyhow::Result<NodePtr> {
    Ok(match member {
        VaultMember::Bls { public_key, .. } => ctx.curry(BlsMember::new(public_key))?,
        VaultMember::Secp256r1 { public_key, .. } => {
            ctx.curry(R1MemberPuzzleAssert::new(public_key))?
        }
        VaultMember::Secp256k1 { public_key, .. } => {
            ctx.curry(K1MemberPuzzleAssert::new(public_key))?
        }
        VaultMember::Passkey { public_key, .. } => {
            ctx.curry(PasskeyMemberPuzzleAssert::new(public_key))?
        }
        _ => anyhow::bail!("unsupported member"),
    })
}

/// Spends a vault through its custody, signed by the given members, and returns the vault's launcher
/// id with what the spend reveals.
fn custody_spend(
    config: &VaultConfig,
    signers: &[VaultMember],
) -> anyhow::Result<(Bytes32, Program, Program)> {
    let mut ctx = SpendContext::new();
    let hashes = vault_hashes(config)?;

    let (_conditions, vault) = Launcher::new(Bytes32::new([1; 32]), 1).mint_vault(
        &mut ctx,
        hashes.custody_hash,
        NodePtr::NIL,
    )?;

    let delegated_spend = ctx.delegated_spend(Conditions::new())?;
    let mut spend = MipsSpend::new(delegated_spend);
    spend.members.insert(
        hashes.custody_hash,
        InnerPuzzleSpend::m_of_n(
            0,
            vec![],
            1,
            vec![hashes.custody_member_hash, hashes.recovery_member_hash],
        ),
    );

    let wrapped =
        |member: VaultMember| mips_puzzle_hash(0, vec![], member_puzzle_hash(member), false);

    if let [member] = config.custody.members.as_slice() {
        let puzzle = member_puzzle(&mut ctx, *member)?;
        spend.members.insert(
            hashes.custody_member_hash,
            InnerPuzzleSpend::new(0, vec![], Spend::new(puzzle, NodePtr::NIL)),
        );
    } else {
        let mut items: Vec<TreeHash> = config
            .custody
            .members
            .iter()
            .copied()
            .map(wrapped)
            .collect();
        items.sort();

        spend.members.insert(
            hashes.custody_member_hash,
            InnerPuzzleSpend::m_of_n(0, vec![], config.custody.threshold, items),
        );

        for &member in signers {
            let puzzle = member_puzzle(&mut ctx, member)?;
            spend.members.insert(
                wrapped(member),
                InnerPuzzleSpend::new(0, vec![], Spend::new(puzzle, NodePtr::NIL)),
            );
        }
    }

    vault.spend(&mut ctx, &spend)?;

    let coin_spend = ctx
        .take()
        .into_iter()
        .find(|coin_spend| coin_spend.coin == vault.coin)
        .ok_or_else(|| anyhow::anyhow!("the vault wasn't spent"))?;

    Ok((
        vault.info.launcher_id,
        coin_spend.puzzle_reveal,
        coin_spend.solution,
    ))
}

#[test]
fn test_rebuild_single_custody() -> anyhow::Result<()> {
    let custody = BlsPair::new(1);
    let recovery = BlsPair::new(2);

    // A timelock that isn't the default, but is a whole number of minutes.
    let config = bls_config(
        VaultSigners::new(vec![VaultMember::bls(custody.pk)], 1),
        recovery.pk,
        2 * HOUR + 30 * MINUTE,
    );

    let (launcher_id, puzzle, solution) = custody_spend(&config, &[])?;
    let reveal = parse_vault_spend(launcher_id, &puzzle, &solution).expect("custody is revealed");

    assert_eq!(
        reveal.custody_hash,
        Bytes32::from(vault_hashes(&config)?.custody_hash)
    );
    assert_eq!(reveal.signers(), Some(config.custody.clone()));

    // The spend belongs to this vault only.
    assert!(parse_vault_spend(Bytes32::new([9; 32]), &puzzle, &solution).is_none());

    let members = recovery_key_members(RecoveryKey::Bls(recovery.pk));
    let found = find_recovery(&reveal, &members, &common_recovery_timelocks())
        .expect("the recovery key matches");
    assert_eq!(found.member, VaultMember::bls(recovery.pk));
    assert_eq!(found.recovery_timelock, config.recovery_timelock);
    assert_eq!(found.version, config.version);

    // Another key doesn't match.
    let other = recovery_key_members(RecoveryKey::Bls(BlsPair::new(3).pk));
    assert_eq!(
        find_recovery(&reveal, &other, &common_recovery_timelocks()[..1000]),
        None
    );

    Ok(())
}

#[test]
fn test_rebuild_uncommon_timelock() -> anyhow::Result<()> {
    let custody = BlsPair::new(1);
    let recovery = BlsPair::new(2);

    let mut config = bls_config(
        VaultSigners::new(vec![VaultMember::bls(custody.pk)], 1),
        recovery.pk,
        61,
    );
    config.version = VaultVersion::Legacy;

    let (launcher_id, puzzle, solution) = custody_spend(&config, &[])?;
    let reveal = parse_vault_spend(launcher_id, &puzzle, &solution).expect("custody is revealed");
    let members = recovery_key_members(RecoveryKey::Bls(recovery.pk));

    // A timelock that isn't a whole number of minutes must be given.
    assert_eq!(
        find_recovery(&reveal, &members, &[DEFAULT_RECOVERY_TIMELOCK]),
        None
    );

    let found = find_recovery(&reveal, &members, &[61]).expect("the timelock matches");
    assert_eq!(found.version, VaultVersion::Legacy);

    Ok(())
}

#[test]
fn test_rebuild_secp_recovery_key() -> anyhow::Result<()> {
    let custody = R1Pair::new(1);
    let recovery = K1Pair::new(2);

    let config = VaultConfig {
        custody: VaultSigners::new(vec![VaultMember::passkey(custody.pk)], 1),
        recovery: VaultSigners::new(vec![VaultMember::secp256k1(recovery.pk)], 1),
        recovery_timelock: DEFAULT_RECOVERY_TIMELOCK,
        version: VaultVersion::ForceSingletonRecreation,
    };

    let (launcher_id, puzzle, solution) = custody_spend(&config, &[])?;
    let reveal = parse_vault_spend(launcher_id, &puzzle, &solution).expect("custody is revealed");
    assert_eq!(reveal.signers(), Some(config.custody.clone()));

    // A compressed secp256k1 key could be either curve, so both are tried.
    let members = recovery_key_members(RecoveryKey::Secp(recovery.pk.to_bytes()));
    let found = find_recovery(&reveal, &members, &common_recovery_timelocks())
        .expect("the recovery key matches");
    assert_eq!(found.member, VaultMember::secp256k1(recovery.pk));

    Ok(())
}

#[test]
fn test_rebuild_m_of_n_custody() -> anyhow::Result<()> {
    let keys = BlsPair::range_vec_with_seed(10, 3);
    let members: Vec<VaultMember> = keys.iter().map(|key| VaultMember::bls(key.pk)).collect();
    let recovery = BlsPair::new(2);

    let config = bls_config(
        VaultSigners::new(members.clone(), 2),
        recovery.pk,
        DEFAULT_RECOVERY_TIMELOCK,
    );

    // A spend only reveals the members that signed it.
    let (launcher_id, puzzle, solution) = custody_spend(&config, &members[..2])?;
    let mut reveal =
        parse_vault_spend(launcher_id, &puzzle, &solution).expect("custody is revealed");
    assert_eq!(reveal.threshold, Some(2));
    assert_eq!(reveal.members.len(), 2);
    assert_eq!(reveal.signers(), None);

    // The recovery side doesn't depend on the hidden members.
    let recovery_members = recovery_key_members(RecoveryKey::Bls(recovery.pk));
    assert!(find_recovery(&reveal, &recovery_members, &[DEFAULT_RECOVERY_TIMELOCK]).is_some());

    // Once every member has signed a spend, the custody is known.
    let (_, puzzle, solution) = custody_spend(&config, &members[1..])?;
    let other = parse_vault_spend(launcher_id, &puzzle, &solution).expect("custody is revealed");
    reveal.merge(&other);

    let signers = reveal.signers().expect("every member is revealed");
    assert_eq!(signers.threshold, 2);
    assert_eq!(signers_hash(&signers, vec![])?, reveal.custody_member_hash);

    Ok(())
}

#[test]
fn test_recovery_hasher() {
    let custody_member_hash = TreeHash::new([3; 32]);
    let member_hash = member_puzzle_hash(VaultMember::bls(BlsPair::new(2).pk));

    for version in [VaultVersion::ForceSingletonRecreation, VaultVersion::Legacy] {
        let hasher = RecoveryHasher::new(custody_member_hash, version);

        for timelock in [1, 60, DEFAULT_RECOVERY_TIMELOCK, 3650 * DAY, u64::MAX] {
            let expected = mips_puzzle_hash(
                0,
                recovery_restrictions(custody_member_hash, timelock, version),
                member_hash,
                false,
            );

            assert_eq!(
                hasher.member_hash(hasher.validator(timelock), hasher.member_tail(member_hash)),
                expected
            );
        }
    }
}

#[test]
fn test_common_recovery_timelocks() {
    let timelocks = common_recovery_timelocks();

    assert_eq!(timelocks[0], DEFAULT_RECOVERY_TIMELOCK);
    assert!(timelocks.contains(&(7 * DAY - MINUTE)));
    assert!(timelocks.contains(&(365 * DAY - HOUR)));
    assert!(timelocks.contains(&(3650 * DAY)));
    assert!(!timelocks.contains(&(7 * DAY + MINUTE)));

    let unique: HashSet<u64> = timelocks.iter().copied().collect();
    assert_eq!(unique.len(), timelocks.len());
}

/// Times a search through every common timelock which doesn't match, which is the slowest case.
#[test]
#[ignore = "slow without optimizations"]
fn bench_full_recovery_search() -> anyhow::Result<()> {
    let config = bls_config(
        VaultSigners::new(vec![VaultMember::bls(BlsPair::new(1).pk)], 1),
        BlsPair::new(2).pk,
        DEFAULT_RECOVERY_TIMELOCK,
    );

    let (launcher_id, puzzle, solution) = custody_spend(&config, &[])?;
    let reveal = parse_vault_spend(launcher_id, &puzzle, &solution).expect("custody is revealed");
    let other = recovery_key_members(RecoveryKey::Bls(BlsPair::new(3).pk));

    let start = std::time::Instant::now();
    assert_eq!(
        find_recovery(&reveal, &other, &common_recovery_timelocks()),
        None
    );
    println!("searched every common timelock in {:?}", start.elapsed());

    Ok(())
}
