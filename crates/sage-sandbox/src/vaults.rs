//! Vaults on the simulated chain, for testing vault import and the vault page.
//!
//! The sandbox mints a fixed set of vaults at startup, one for each way a vault can be imported,
//! including ones Sage doesn't mint itself, such as legacy vaults and vaults without memos. Alice's
//! key is the custody of each, so she can attach it as a signer, and bob's key is the recovery key
//! where there is one.

use std::{fs, path::Path, slice};

use anyhow::Result;
use bip39::Mnemonic;
use chia_wallet_sdk::{
    driver::{InnerPuzzleSpend, MipsSpend, P2Singleton, Vault, VaultVersion},
    prelude::*,
    test::{PeerSimulator, Simulator},
    types::{
        MessageFlags, MessageSide, Mod,
        puzzles::{
            BlsMember, Force1of2RestrictedVariable, Force1of2RestrictedVariableSolution,
            ForceSingletonRecreationMod, PreventConditionOpcode, PreventMultipleCreateCoinsMod,
            Timelock,
        },
    },
    utils::Address,
};
use sage_database::{VaultConfig, VaultMember, VaultRecoveryConfig, VaultSigners};
use sage_wallet::{
    VaultHashes, build_vault, build_vault_recovery, recovery_restrictions, vault_config_file,
    vault_hashes,
};
use serde::Serialize;
use tracing::info;

/// The recovery timelock of the vaults that have recovery, in seconds. This is a day, so a started
/// recovery stays pending while the sandbox runs.
const TIMELOCK: u64 = 86_400;

/// How much XCH each vault holds at its first address, in mojos. This is 1 XCH.
const VAULT_BALANCE: u64 = 1_000_000_000_000;

/// What agents and people need to know about a vault to import it. This is written to the ready
/// file.
#[derive(Debug, Clone, Serialize)]
pub struct VaultInfo {
    pub name: String,
    /// What the vault is for.
    pub description: String,
    pub launcher_id: String,
    /// The vault's first address, which it has spent from, so the vault can be found from it.
    pub address: String,
    /// Whether the vault's config is in its memos, so it can be imported from its id alone.
    pub memo: bool,
    /// The recovery keys to import it with if it has no memos, as master public keys in hex.
    pub recovery_keys: Vec<String>,
    /// The recovery threshold, if there's more than one key.
    pub recovery_threshold: Option<usize>,
    /// A config file for the vault, in the format Sage and Chia Cloud Wallet export.
    pub config_file: String,
}

/// Mints the sandbox's vaults, writes a config file for each to `dir`, and returns their details.
pub async fn mint_vaults(
    sim: &PeerSimulator,
    dir: &Path,
    custody_seed: u8,
    recovery_seed: u8,
) -> Result<Vec<VaultInfo>> {
    let custody = master_key(custody_seed)?;
    let recovery = master_key(recovery_seed)?;
    // A second recovery key that isn't any wallet's, for a vault with a 2 of 2 recovery.
    let other = master_key(100)?;

    let alice = || VaultSigners::new(vec![VaultMember::bls(custody.public_key())], 1);
    let bob = || VaultSigners::new(vec![VaultMember::bls(recovery.public_key())], 1);

    let fixtures = [
        Fixture {
            name: "memo",
            description: "Has its config in its memos, so its launcher id or address is enough",
            config: VaultConfig::with_recovery(
                alice(),
                bob(),
                TIMELOCK,
                VaultVersion::ForceSingletonRecreation,
            ),
            memo: true,
            recovering: false,
        },
        Fixture {
            name: "no-memo",
            description: "A legacy vault without memos, imported with bob's recovery key",
            config: VaultConfig::with_recovery(alice(), bob(), TIMELOCK, VaultVersion::Legacy),
            memo: false,
            recovering: false,
        },
        Fixture {
            name: "two-recovery-keys",
            description: "Without memos, with a 2 of 2 recovery of bob's key and another key",
            config: VaultConfig::with_recovery(
                alice(),
                VaultSigners::new(
                    vec![
                        VaultMember::bls(recovery.public_key()),
                        VaultMember::bls(other.public_key()),
                    ],
                    2,
                ),
                TIMELOCK,
                VaultVersion::ForceSingletonRecreation,
            ),
            memo: false,
            recovering: false,
        },
        Fixture {
            name: "no-recovery",
            description: "Without memos or recovery, so it's rebuilt from its spends alone",
            config: VaultConfig::without_recovery(alice()),
            memo: false,
            recovering: false,
        },
        Fixture {
            name: "recovering",
            description: "Bob has started recovery, which alice's custody can cancel",
            config: VaultConfig::with_recovery(
                alice(),
                bob(),
                TIMELOCK,
                VaultVersion::ForceSingletonRecreation,
            ),
            memo: true,
            recovering: true,
        },
    ];

    let mut sim = sim.lock().await;
    let vault_dir = dir.join("vaults");
    fs::create_dir_all(&vault_dir)?;

    let mut vaults = Vec::new();

    for fixture in fixtures {
        let vault = mint(&mut sim, &fixture.config, fixture.memo)?;
        let launcher_id = vault.info.launcher_id;
        let vault = spend_from_address(&mut sim, &vault, &fixture.config, &custody)?;

        if fixture.recovering {
            let post_recovery = VaultConfig::with_recovery(
                bob(),
                alice(),
                TIMELOCK,
                VaultVersion::ForceSingletonRecreation,
            );
            start_recovery(&mut sim, &vault, &fixture.config, &recovery, &post_recovery)?;
        }

        let address = P2Singleton::new(launcher_id, 0).tree_hash().into();
        sim.new_coin(address, VAULT_BALANCE);

        let config_file = vault_dir.join(format!("{}.json", fixture.name));
        fs::write(
            &config_file,
            vault_config_file(launcher_id, &fixture.config)?,
        )?;

        let recovery_signers = fixture
            .config
            .recovery
            .as_ref()
            .map(|recovery| &recovery.signers);

        let info = VaultInfo {
            name: fixture.name.to_string(),
            description: fixture.description.to_string(),
            launcher_id: Address::new(launcher_id, "vault".to_string()).encode()?,
            address: Address::new(address, "txch".to_string()).encode()?,
            memo: fixture.memo,
            recovery_keys: recovery_signers
                .map(|signers| {
                    signers
                        .members
                        .iter()
                        .filter_map(|member| match member {
                            VaultMember::Bls { public_key, .. } => {
                                Some(hex::encode(public_key.to_bytes()))
                            }
                            _ => None,
                        })
                        .collect()
                })
                .unwrap_or_default(),
            recovery_threshold: recovery_signers
                .filter(|signers| signers.members.len() > 1)
                .map(|signers| signers.threshold),
            config_file: config_file.display().to_string(),
        };

        info!("Minted the {} vault {}", info.name, info.launcher_id);
        vaults.push(info);
    }

    Ok(vaults)
}

struct Fixture {
    name: &'static str,
    description: &'static str,
    config: VaultConfig,
    memo: bool,
    recovering: bool,
}

/// The master secret key of the sandbox wallet with this seed byte.
fn master_key(seed: u8) -> Result<SecretKey> {
    let mnemonic = Mnemonic::from_entropy(&[seed; 16])?;
    Ok(SecretKey::from_seed(&mnemonic.to_seed("")))
}

/// Mints a vault, with its memo in the launcher solution unless `memo` is false.
fn mint(sim: &mut Simulator, config: &VaultConfig, memo: bool) -> Result<Vault> {
    let funder = sim.bls(1);

    let mut ctx = SpendContext::new();
    let built = build_vault(&mut ctx, config)?;
    let memos = if memo { built.memo_list } else { NodePtr::NIL };

    let (conditions, vault) = Launcher::new(funder.coin.coin_id(), 1).mint_vault(
        &mut ctx,
        built.hashes.custody_hash,
        memos,
    )?;
    StandardLayer::new(funder.pk).spend(&mut ctx, funder.coin, conditions)?;
    sim.spend_coins(ctx.take(), &[funder.sk])?;

    Ok(vault)
}

/// Spends a coin from the vault's first address, which the vault approves with a message, and
/// recreates the vault without a memo. This reveals the address's puzzle, so the vault can be found
/// from it, and the vault's custody, so its config can be rebuilt.
fn spend_from_address(
    sim: &mut Simulator,
    vault: &Vault,
    config: &VaultConfig,
    custody: &SecretKey,
) -> Result<Vault> {
    let mut ctx = SpendContext::new();

    let p2_singleton = P2Singleton::new(vault.info.launcher_id, 0);
    let coin = sim.new_coin(p2_singleton.tree_hash().into(), 10);

    let delegated_spend =
        ctx.delegated_spend(Conditions::new().create_coin(Bytes32::default(), 10, Memos::None))?;
    let delegated_puzzle_hash = ctx.tree_hash(delegated_spend.puzzle);
    let spend = p2_singleton.spend(
        &mut ctx,
        vault.info.custody_hash.into(),
        vault.coin.amount,
        delegated_spend,
    )?;
    ctx.spend(coin, spend)?;

    let coin_id = ctx.alloc(&coin.coin_id())?;
    let hashes = vault_hashes(config)?;
    let conditions = Conditions::new()
        .create_coin(
            vault.info.custody_hash.into(),
            vault.coin.amount,
            Memos::None,
        )
        .send_message(
            MessageFlags::PUZZLE.encode(MessageSide::Sender)
                | MessageFlags::COIN.encode(MessageSide::Receiver),
            delegated_puzzle_hash.to_vec().into(),
            vec![coin_id],
        );

    let mut spend = MipsSpend::new(ctx.delegated_spend(conditions)?);
    let member = ctx.curry(BlsMember::new(custody.public_key()))?;
    let member = InnerPuzzleSpend::new(0, vec![], Spend::new(member, NodePtr::NIL));

    // A vault without recovery is spent through its custody at the top level.
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
        spend.members.insert(hashes.custody_member_hash, member);
    } else {
        spend.members.insert(hashes.custody_hash, member);
    }
    vault.spend(&mut ctx, &spend)?;

    sim.spend_coins(ctx.take(), slice::from_ref(custody))?;

    Ok(vault.child(vault.info.custody_hash, vault.coin.amount))
}

/// Starts recovery with the recovery key, moving the vault to the recovery state, which moves it to
/// `post_recovery` once the timelock expires.
fn start_recovery(
    sim: &mut Simulator,
    vault: &Vault,
    config: &VaultConfig,
    recovery: &SecretKey,
    post_recovery: &VaultConfig,
) -> Result<()> {
    let mut ctx = SpendContext::new();
    let hashes = vault_hashes(config)?;
    let recovery_side = config
        .recovery
        .as_ref()
        .expect("a recovering vault has recovery");
    let recovery_member_hash = recovery_member_hash(hashes);

    let recovery_config = VaultRecoveryConfig {
        custody: config.custody.clone(),
        recovery_timelock: recovery_side.timelock,
        version: recovery_side.version,
        recovery_custody_hash: vault_hashes(post_recovery)?.custody_hash.into(),
        amount: vault.coin.amount,
    };
    let built = build_vault_recovery(&mut ctx, &recovery_config, post_recovery)?;

    // Force singleton recreation requires the vault to assert its amount, which it recreates.
    let conditions = Conditions::new()
        .create_coin(
            built.hashes.custody_hash.into(),
            vault.coin.amount,
            Memos::Some(built.memo_list),
        )
        .assert_my_amount(vault.coin.amount);
    let mut spend = MipsSpend::new(ctx.delegated_spend(conditions)?);

    spend.members.insert(
        hashes.custody_hash,
        InnerPuzzleSpend::m_of_n(
            0,
            vec![],
            1,
            vec![hashes.custody_member_hash, recovery_member_hash],
        ),
    );
    let member = ctx.curry(BlsMember::new(recovery.public_key()))?;
    spend.members.insert(
        recovery_member_hash,
        InnerPuzzleSpend::new(
            0,
            recovery_restrictions(
                hashes.custody_member_hash,
                recovery_side.timelock,
                recovery_side.version,
            ),
            Spend::new(member, NodePtr::NIL),
        ),
    );

    // The recovery path's delegated puzzle wrappers, in order.
    let force_1_of_2 = Force1of2RestrictedVariable::new(
        hashes.custody_member_hash.into(),
        0,
        vec![Timelock::new(recovery_side.timelock).curry_tree_hash()]
            .tree_hash()
            .into(),
        ().tree_hash().into(),
    );
    let puzzle = ctx.curry(force_1_of_2)?;
    // The wrapper rebuilds the timelocked right side from the finish member's puzzle hash.
    let finish_puzzle = built
        .recovery_finish_puzzle
        .expect("a recovery state has a finish puzzle");
    let finish_puzzle_hash = ctx.tree_hash(finish_puzzle).into();
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

    if recovery_side.version == VaultVersion::ForceSingletonRecreation {
        let puzzle = ctx.alloc_mod::<ForceSingletonRecreationMod>()?;
        spend.restrictions.insert(
            ForceSingletonRecreationMod::mod_hash(),
            Spend::new(puzzle, NodePtr::NIL),
        );
    }

    vault.spend(&mut ctx, &spend)?;
    sim.spend_coins(ctx.take(), slice::from_ref(recovery))?;

    Ok(())
}

fn recovery_member_hash(hashes: VaultHashes) -> TreeHash {
    hashes.recovery_member_hash.expect("the vault has recovery")
}
