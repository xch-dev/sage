use chia_wallet_sdk::{
    driver::{InnerPuzzleSpend, MipsSpend, mips_puzzle_hash},
    test::{BlsPair, K1Pair, R1Pair},
    types::puzzles::{
        BlsMember, K1MemberPuzzleAssert, PasskeyMemberPuzzleAssert, R1MemberPuzzleAssert,
    },
};

use super::*;

fn bls_config(custody: VaultSigners, recovery: PublicKey, timelock: u64) -> VaultConfig {
    VaultConfig::with_recovery(
        custody,
        VaultSigners::new(vec![VaultMember::bls(recovery)], 1),
        timelock,
        VaultVersion::ForceSingletonRecreation,
    )
}

fn single(member: VaultMember) -> VaultSigners {
    VaultSigners::new(vec![member], 1)
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

/// Adds the spend of a custody, signed by the given members, at `hash`.
fn insert_custody(
    ctx: &mut SpendContext,
    spend: &mut MipsSpend,
    hash: TreeHash,
    custody: &VaultSigners,
    signers: &[VaultMember],
) -> anyhow::Result<()> {
    let wrapped =
        |member: VaultMember| mips_puzzle_hash(0, vec![], member_puzzle_hash(member), false);

    if let [member] = custody.members.as_slice() {
        let puzzle = member_puzzle(ctx, *member)?;
        spend.members.insert(
            hash,
            InnerPuzzleSpend::new(0, vec![], Spend::new(puzzle, NodePtr::NIL)),
        );
        return Ok(());
    }

    let mut items: Vec<TreeHash> = custody.members.iter().copied().map(wrapped).collect();
    items.sort();

    spend.members.insert(
        hash,
        InnerPuzzleSpend::m_of_n(0, vec![], custody.threshold, items),
    );

    for &member in signers {
        let puzzle = member_puzzle(ctx, member)?;
        spend.members.insert(
            wrapped(member),
            InnerPuzzleSpend::new(0, vec![], Spend::new(puzzle, NodePtr::NIL)),
        );
    }

    Ok(())
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

    if let Some(recovery_member_hash) = hashes.recovery_member_hash {
        spend.members.insert(
            hashes.custody_hash,
            InnerPuzzleSpend::m_of_n(
                0,
                vec![],
                1,
                vec![hashes.custody_member_hash, recovery_member_hash],
            ),
        );
        insert_custody(
            &mut ctx,
            &mut spend,
            hashes.custody_member_hash,
            &config.custody,
            signers,
        )?;
    } else {
        insert_custody(
            &mut ctx,
            &mut spend,
            hashes.custody_hash,
            &config.custody,
            signers,
        )?;
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

fn search(keys: &[RecoveryKey], timelocks: Vec<u64>) -> RecoverySearch {
    RecoverySearch::new(keys, None, timelocks).expect("the search is valid")
}

fn find(reveal: &CustodyReveal, search: &RecoverySearch) -> Option<RecoveryMatch> {
    let custody_side = reveal.with_recovery.as_ref()?;
    find_recovery(custody_side.hash, reveal.top_level_hash, search)
}

#[test]
fn test_rebuild_single_custody() -> anyhow::Result<()> {
    let custody = BlsPair::new(1);
    let recovery = BlsPair::new(2);

    // A timelock that isn't the default, but is a whole number of minutes.
    let config = bls_config(
        single(VaultMember::bls(custody.pk)),
        recovery.pk,
        2 * HOUR + 30 * MINUTE,
    );

    let (launcher_id, puzzle, solution) = custody_spend(&config, &[])?;
    let reveal = parse_vault_spend(launcher_id, &puzzle, &solution).expect("custody is revealed");

    assert_eq!(
        reveal.custody_hash,
        Bytes32::from(vault_hashes(&config)?.custody_hash)
    );
    assert_eq!(reveal.custody_with_recovery(), Some(config.custody.clone()));

    // It could also be a 1 of N custody without recovery, but the other member isn't known.
    assert!(reveal.without_recovery.is_some());
    assert_eq!(reveal.custody_without_recovery(), None);

    // The spend belongs to this vault only.
    assert!(parse_vault_spend(Bytes32::new([9; 32]), &puzzle, &solution).is_none());

    let found = find(
        &reveal,
        &search(
            &[RecoveryKey::Bls(recovery.pk)],
            common_recovery_timelocks(),
        ),
    )
    .expect("the recovery key matches");
    assert_eq!(found.signers, single(VaultMember::bls(recovery.pk)));
    assert_eq!(found.recovery_timelock, 2 * HOUR + 30 * MINUTE);
    assert_eq!(found.version, VaultVersion::ForceSingletonRecreation);

    // Another key doesn't match.
    let other = search(
        &[RecoveryKey::Bls(BlsPair::new(3).pk)],
        common_recovery_timelocks()[..1000].to_vec(),
    );
    assert_eq!(find(&reveal, &other), None);

    Ok(())
}

#[test]
fn test_rebuild_uncommon_timelock() -> anyhow::Result<()> {
    let custody = BlsPair::new(1);
    let recovery = BlsPair::new(2);

    let config = VaultConfig::with_recovery(
        single(VaultMember::bls(custody.pk)),
        single(VaultMember::bls(recovery.pk)),
        61,
        VaultVersion::Legacy,
    );

    let (launcher_id, puzzle, solution) = custody_spend(&config, &[])?;
    let reveal = parse_vault_spend(launcher_id, &puzzle, &solution).expect("custody is revealed");
    let keys = [RecoveryKey::Bls(recovery.pk)];

    // A timelock that isn't a whole number of minutes must be given.
    assert_eq!(
        find(&reveal, &search(&keys, vec![DEFAULT_RECOVERY_TIMELOCK])),
        None
    );

    let found = find(&reveal, &search(&keys, vec![61])).expect("the timelock matches");
    assert_eq!(found.version, VaultVersion::Legacy);

    Ok(())
}

#[test]
fn test_rebuild_secp_recovery_key() -> anyhow::Result<()> {
    let custody = R1Pair::new(1);
    let recovery = K1Pair::new(2);

    let config = VaultConfig::with_recovery(
        single(VaultMember::passkey(custody.pk)),
        single(VaultMember::secp256k1(recovery.pk)),
        DEFAULT_RECOVERY_TIMELOCK,
        VaultVersion::ForceSingletonRecreation,
    );

    let (launcher_id, puzzle, solution) = custody_spend(&config, &[])?;
    let reveal = parse_vault_spend(launcher_id, &puzzle, &solution).expect("custody is revealed");
    assert_eq!(reveal.custody_with_recovery(), Some(config.custody.clone()));

    // A compressed secp256k1 key could be either curve, so both are tried.
    let found = find(
        &reveal,
        &search(
            &[RecoveryKey::Secp(recovery.pk.to_bytes())],
            common_recovery_timelocks(),
        ),
    )
    .expect("the recovery key matches");
    assert_eq!(found.signers, single(VaultMember::secp256k1(recovery.pk)));

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
    let custody_side = reveal.with_recovery.clone().expect("custody is revealed");
    assert_eq!(custody_side.threshold, Some(2));
    assert_eq!(custody_side.members.len(), 2);
    assert_eq!(reveal.custody_with_recovery(), None);

    // An M of N custody side can't be the top level of a vault without recovery.
    assert_eq!(reveal.without_recovery, None);

    // The recovery side doesn't depend on the hidden members.
    let keys = [RecoveryKey::Bls(recovery.pk)];
    assert!(find(&reveal, &search(&keys, vec![DEFAULT_RECOVERY_TIMELOCK])).is_some());

    // Once every member has signed a spend, the custody is known.
    let (_, puzzle, solution) = custody_spend(&config, &members[1..])?;
    let other = parse_vault_spend(launcher_id, &puzzle, &solution).expect("custody is revealed");
    reveal.merge(&other);

    let signers = reveal
        .custody_with_recovery()
        .expect("every member is revealed");
    assert_eq!(signers.threshold, 2);
    assert_eq!(signers_hash(&signers, vec![])?, custody_side.hash);

    Ok(())
}

#[test]
fn test_rebuild_multiple_recovery_keys() -> anyhow::Result<()> {
    let bls = BlsPair::new(2);
    let r1 = R1Pair::new(3);
    let k1 = K1Pair::new(4);

    let recovery = VaultSigners::new(
        vec![
            VaultMember::bls(bls.pk),
            VaultMember::passkey(r1.pk),
            VaultMember::secp256k1(k1.pk),
        ],
        2,
    );
    let config = VaultConfig::with_recovery(
        single(VaultMember::bls(BlsPair::new(1).pk)),
        recovery.clone(),
        3 * DAY,
        VaultVersion::ForceSingletonRecreation,
    );

    let (launcher_id, puzzle, solution) = custody_spend(&config, &[])?;
    let reveal = parse_vault_spend(launcher_id, &puzzle, &solution).expect("custody is revealed");

    // The keys can be in any order, and the threshold is searched for.
    let keys = [
        RecoveryKey::Secp(k1.pk.to_bytes()),
        RecoveryKey::Bls(bls.pk),
        RecoveryKey::Secp(r1.pk.to_bytes()),
    ];
    let found = find(&reveal, &search(&keys, common_recovery_timelocks()))
        .expect("the recovery keys match");
    assert_eq!(found.recovery_timelock, 3 * DAY);
    assert_eq!(found.signers.threshold, 2);
    assert_eq!(
        signers_hash(&found.signers, vec![])?,
        signers_hash(&recovery, vec![])?
    );

    // A threshold that's given is the only one tried.
    let wrong_threshold = RecoverySearch::new(&keys, Some(1), vec![3 * DAY])?;
    assert_eq!(find(&reveal, &wrong_threshold), None);

    let threshold = RecoverySearch::new(&keys, Some(2), vec![3 * DAY])?;
    assert!(find(&reveal, &threshold).is_some());

    // Every recovery key must be given.
    assert_eq!(find(&reveal, &search(&keys[..2], vec![3 * DAY])), None);

    Ok(())
}

#[test]
fn test_recovery_search_limits() {
    let keys = [RecoveryKey::Bls(BlsPair::new(1).pk)];

    assert!(matches!(
        RecoverySearch::new(&keys, Some(0), vec![1]),
        Err(VaultHistoryError::InvalidRecoveryThreshold)
    ));
    assert!(matches!(
        RecoverySearch::new(&keys, Some(2), vec![1]),
        Err(VaultHistoryError::InvalidRecoveryThreshold)
    ));

    // A lone key is tried as either of its members, and repeated keys are only tried once.
    let search = RecoverySearch::new(&[keys[0], keys[0]], None, vec![1]).unwrap();
    assert_eq!(search.candidates.len(), 2);

    // Several keys are tried as Cloud Wallet's members, with every threshold.
    let several: Vec<RecoveryKey> = (1..=3)
        .map(|seed| RecoveryKey::Bls(BlsPair::new(seed).pk))
        .collect();
    let search = RecoverySearch::new(&several, None, vec![1]).unwrap();
    assert_eq!(search.candidates.len(), 3);

    // A search that's too large must be narrowed down with a threshold or timelock.
    let timelocks = vec![1; MAX_RECOVERY_SEARCH / 2];
    assert!(matches!(
        RecoverySearch::new(&several, None, timelocks.clone()),
        Err(VaultHistoryError::RecoverySearchTooLarge)
    ));
    assert!(RecoverySearch::new(&several, Some(2), timelocks).is_ok());
}

#[test]
fn test_rebuild_without_recovery() -> anyhow::Result<()> {
    // A single member is the whole top level.
    let custody = single(VaultMember::bls(BlsPair::new(1).pk));
    let config = VaultConfig::without_recovery(custody.clone());

    let (launcher_id, puzzle, solution) = custody_spend(&config, &[])?;
    let reveal = parse_vault_spend(launcher_id, &puzzle, &solution).expect("custody is revealed");
    assert_eq!(
        reveal.custody_hash,
        Bytes32::from(vault_hashes(&config)?.custody_hash)
    );
    assert_eq!(reveal.with_recovery, None);
    assert_eq!(reveal.custody_without_recovery(), Some(custody));

    // An M of N is known once every member has signed.
    let keys = BlsPair::range_vec_with_seed(20, 3);
    let members: Vec<VaultMember> = keys.iter().map(|key| VaultMember::bls(key.pk)).collect();
    let config = VaultConfig::without_recovery(VaultSigners::new(members.clone(), 2));

    let (launcher_id, puzzle, solution) = custody_spend(&config, &members[..2])?;
    let mut reveal =
        parse_vault_spend(launcher_id, &puzzle, &solution).expect("custody is revealed");
    assert_eq!(reveal.with_recovery, None);
    assert_eq!(reveal.custody_without_recovery(), None);
    assert_eq!(reveal.revealed(), 2);

    let (_, puzzle, solution) = custody_spend(&config, &members[1..])?;
    reveal.merge(&parse_vault_spend(launcher_id, &puzzle, &solution).expect("custody is revealed"));
    assert_eq!(
        reveal
            .custody_without_recovery()
            .map(|signers| signers_inner_hash(&signers)),
        Some(signers_inner_hash(&config.custody))
    );

    // A 1 of N looks like a vault with recovery, until another member signs.
    let config = VaultConfig::without_recovery(VaultSigners::new(members[..2].to_vec(), 1));

    let (launcher_id, puzzle, solution) = custody_spend(&config, &members[..1])?;
    let mut reveal =
        parse_vault_spend(launcher_id, &puzzle, &solution).expect("custody is revealed");
    assert!(reveal.with_recovery.is_some());
    assert_eq!(reveal.custody_without_recovery(), None);

    let (_, puzzle, solution) = custody_spend(&config, &members[1..2])?;
    reveal.merge(&parse_vault_spend(launcher_id, &puzzle, &solution).expect("custody is revealed"));
    assert!(reveal.custody_without_recovery().is_some());

    Ok(())
}

#[test]
fn test_recovery_hasher() -> anyhow::Result<()> {
    let custody_member_hash = TreeHash::new([3; 32]);
    let recovery_signers = [
        single(VaultMember::bls(BlsPair::new(2).pk)),
        VaultSigners::new(
            vec![
                VaultMember::bls(BlsPair::new(4).pk),
                VaultMember::secp256r1(R1Pair::new(5).pk),
            ],
            2,
        ),
    ];

    for version in [VaultVersion::ForceSingletonRecreation, VaultVersion::Legacy] {
        let hasher = RecoveryHasher::new(custody_member_hash, version);

        for signers in &recovery_signers {
            for timelock in [1, 60, DEFAULT_RECOVERY_TIMELOCK, 3650 * DAY, u64::MAX] {
                let expected = signers_hash(
                    signers,
                    recovery_restrictions(custody_member_hash, timelock, version),
                )?;

                assert_eq!(
                    hasher.member_hash(
                        hasher.validator(timelock),
                        hasher.member_tail(signers_inner_hash(signers))
                    ),
                    expected
                );
            }
        }
    }

    Ok(())
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
        single(VaultMember::bls(BlsPair::new(1).pk)),
        BlsPair::new(2).pk,
        DEFAULT_RECOVERY_TIMELOCK,
    );

    let (launcher_id, puzzle, solution) = custody_spend(&config, &[])?;
    let reveal = parse_vault_spend(launcher_id, &puzzle, &solution).expect("custody is revealed");

    let secp = |seed: u64| RecoveryKey::Secp(R1Pair::new(seed).pk.to_bytes());
    let bls = |seed: u64| RecoveryKey::Bls(BlsPair::new(seed).pk);

    for keys in [
        vec![secp(100)],
        vec![bls(100), bls(101), bls(102)],
        vec![secp(100), secp(101), secp(102)],
    ] {
        let search = search(&keys, common_recovery_timelocks());

        let start = std::time::Instant::now();
        assert_eq!(find(&reveal, &search), None);
        println!(
            "searched {} candidates with every common timelock in {:?}",
            search.candidates.len(),
            start.elapsed()
        );
    }

    Ok(())
}
