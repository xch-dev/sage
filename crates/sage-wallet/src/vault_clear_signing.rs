//! Clear signing for vault transactions, which checks what a vault spend does before its custody
//! signs it, using the SDK's [`parse_vault_transaction`].
//!
//! The SDK only parses the XCH, CAT, NFT and bulletin spends that a vault approves. A vault's DIDs
//! and options are checked here instead: each one must be locked to the vault's own address, and
//! run the delegated puzzle that the vault approved. Their messages are then left out of the
//! delegated spend that the SDK parses, so it sees the rest of the transaction.

use std::collections::HashMap;

use chia_wallet_sdk::{
    chia::puzzle_types::{did::DidSolution, singleton::SingletonSolution},
    clvm_traits::clvm_quote,
    clvm_utils::tree_hash,
    driver::{
        ClawbackV2, ClearSigningAsset, Did, DidInfo, Layer, OptionContract,
        P2ConditionsOrSingleton, Puzzle, Reveals, SingletonLayer, VAULT_MESSAGE_MODE,
        parse_vault_transaction,
    },
    prelude::*,
    puzzles::SINGLETON_LAUNCHER_HASH,
    types::puzzles::DelegatedPuzzleFeederSolution,
};

use crate::{WalletError, parse_p2_singleton};

/// The largest launcher amount that's counted as part of minting an NFT. The action system gives
/// each launcher that a singleton creates a different even amount, so a transaction that mints
/// many NFTs from a DID may have hidden a few mojos in them.
const MAX_MINTED_LAUNCHER_AMOUNT: u64 = 2;

/// What a vault spend in a transaction does, from the vault's point of view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VaultSpendSummary {
    pub launcher_id: Bytes32,
    /// The vault coin being spent.
    pub coin: Coin,
    /// The vault's custody hash before the spend.
    pub custody_hash: Bytes32,
    /// The custody hash the vault is recreated with, or `None` if it's melted.
    pub child_custody_hash: Option<Bytes32>,
    /// The fee the vault and the spends it approves reserve.
    pub reserved_fee: u64,
    /// The XCH that leaves the vault's approved spends without being accounted for by outputs,
    /// payments or the reserved fee, which would be left as an extra fee.
    pub unaccounted_xch: u128,
    /// The DIDs and options the vault spends, which the SDK doesn't parse.
    pub singletons: Vec<VaultSingletonSpend>,
}

impl VaultSpendSummary {
    /// Whether the spend recreates the vault with a different custody, or melts it.
    pub fn changes_custody(&self) -> bool {
        self.child_custody_hash != Some(self.custody_hash)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VaultSingletonKind {
    Did,
    Option,
}

/// The spend of a DID or option owned by the vault, which the vault approves with a message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VaultSingletonSpend {
    pub kind: VaultSingletonKind,
    pub launcher_id: Bytes32,
    pub coin: Coin,
    /// The p2 puzzle hash the singleton is recreated with, or `None` if it's melted, such as when
    /// an option is exercised.
    pub child_p2_puzzle_hash: Option<Bytes32>,
}

/// What the wallet knows about the coins a vault spends, which clear signing can't tell from the
/// coin spends alone.
#[derive(Debug, Default, Clone)]
pub struct VaultReveals {
    /// The clawbacks that coins being spent are locked to, so their paths can be checked.
    pub clawbacks: Vec<ClawbackV2>,
    /// The P2 conditions or singleton puzzles that coins being spent are locked to, such as
    /// pre-split offer coins.
    pub p2_conditions_or_singletons: Vec<P2ConditionsOrSingleton>,
    /// The nonces of the vault's addresses, so payments to them are counted as received.
    pub nonces: Vec<usize>,
}

/// Finds the spend of the vault with the launcher id in the coin spends, and parses what it does.
/// Returns `None` if the vault isn't spent.
///
/// Only spends through the vault's custody are parsed. A spend through its recovery side, whose
/// delegated puzzle is wrapped, fails to parse.
pub fn parse_vault_spend_bundle(
    coin_spends: &[CoinSpend],
    launcher_id: Bytes32,
) -> Result<Option<VaultSpendSummary>, WalletError> {
    parse_vault_spend_bundle_with(coin_spends, launcher_id, &VaultReveals::default())
}

/// Like [`parse_vault_spend_bundle`], with what the wallet knows about the coins being spent.
pub fn parse_vault_spend_bundle_with(
    coin_spends: &[CoinSpend],
    launcher_id: Bytes32,
    vault_reveals: &VaultReveals,
) -> Result<Option<VaultSpendSummary>, WalletError> {
    let mut ctx = SpendContext::new();

    let mut vault_spend = None;

    for coin_spend in coin_spends {
        let puzzle = coin_spend.puzzle_reveal.to_clvm(&mut ctx)?;
        let puzzle = Puzzle::parse(&ctx, puzzle);

        let Some(singleton) = SingletonLayer::<Puzzle>::parse_puzzle(&ctx, puzzle)? else {
            continue;
        };

        if singleton.launcher_id != launcher_id {
            continue;
        }

        let solution = coin_spend.solution.to_clvm(&mut ctx)?;
        let solution = SingletonLayer::<Puzzle>::parse_solution(&ctx, solution)?;

        let custody_hash: Bytes32 = tree_hash(&ctx, singleton.inner_puzzle.ptr()).into();

        // The index wrapper passes its solution through to the delegated puzzle feeder.
        let feeder = DelegatedPuzzleFeederSolution::<NodePtr, NodePtr, NodePtr>::from_clvm(
            &ctx,
            solution.inner_solution,
        )?;

        vault_spend = Some((
            coin_spend.coin,
            custody_hash,
            Spend::new(feeder.delegated_puzzle, feeder.delegated_solution),
        ));
        break;
    }

    let Some((coin, custody_hash, delegated_spend)) = vault_spend else {
        return Ok(None);
    };

    let coin_spends_by_id: HashMap<Bytes32, &CoinSpend> = coin_spends
        .iter()
        .map(|coin_spend| (coin_spend.coin.coin_id(), coin_spend))
        .collect();

    let (delegated_spend, singletons) =
        split_singleton_spends(&mut ctx, delegated_spend, &coin_spends_by_id, launcher_id)?;

    let mut reveals = Reveals::from_coin_spends(&mut ctx, coin_spends)?;
    reveals.reveal_signing_vault(custody_hash, Some(coin));

    for &clawback in &vault_reveals.clawbacks {
        reveals.reveal_clawback(clawback);
    }

    for &p2 in &vault_reveals.p2_conditions_or_singletons {
        reveals.reveal_p2_conditions_or_singleton(p2);
    }

    for &nonce in &vault_reveals.nonces {
        reveals.reveal_vault_nonce(nonce);
    }

    let transaction = parse_vault_transaction(reveals, &mut ctx, launcher_id, delegated_spend)?;

    let unaccounted_xch: u128 = transaction
        .asset_flows
        .iter()
        .filter(|flow| flow.asset == ClearSigningAsset::Xch)
        .map(|flow| flow.unaccounted_amount)
        .sum();

    // Minting an NFT from the vault or one of its DIDs takes a mojo of XCH for the NFT, which its
    // launcher creates from less, so it's accounted for here.
    let mut minters = vec![coin.coin_id()];
    minters.extend(singletons.iter().map(|singleton| singleton.coin.coin_id()));
    let launched = launcher_amount(&mut ctx, coin_spends, &minters)?;
    let unaccounted_xch = unaccounted_xch.saturating_sub(launched);

    Ok(Some(VaultSpendSummary {
        launcher_id,
        coin,
        custody_hash,
        child_custody_hash: transaction.vault_child.map(|child| child.custody_hash),
        reserved_fee: transaction.reserved_fee,
        unaccounted_xch,
        singletons,
    }))
}

/// Checks the DIDs and options that the vault's delegated spend sends messages to, and returns the
/// delegated spend without those messages, for the SDK to parse the rest of.
fn split_singleton_spends(
    ctx: &mut SpendContext,
    delegated_spend: Spend,
    coin_spends: &HashMap<Bytes32, &CoinSpend>,
    launcher_id: Bytes32,
) -> Result<(Spend, Vec<VaultSingletonSpend>), WalletError> {
    let output = ctx.run(delegated_spend.puzzle, delegated_spend.solution)?;
    let conditions = Vec::<NodePtr>::from_clvm(ctx, output)?;

    let mut kept = Vec::with_capacity(conditions.len());
    let mut singletons = Vec::new();

    for condition in conditions {
        if let Ok(Condition::SendMessage(message)) = Condition::<NodePtr>::from_clvm(ctx, condition)
            && message.mode == VAULT_MESSAGE_MODE
            && let [coin_id] = message.data.as_slice()
            && let Ok(coin_id) = ctx.extract::<Bytes32>(*coin_id)
            && let Some(coin_spend) = coin_spends.get(&coin_id)
            && let Some(singleton) =
                parse_singleton_spend(ctx, coin_spend, &message.message, launcher_id)?
        {
            singletons.push(singleton);
            continue;
        }

        kept.push(condition);
    }

    if singletons.is_empty() {
        return Ok((delegated_spend, singletons));
    }

    let puzzle = ctx.alloc(&clvm_quote!(kept))?;

    Ok((Spend::new(puzzle, NodePtr::NIL), singletons))
}

/// Parses the spend of a DID or option that the vault approves with a message whose content is
/// the delegated puzzle hash. Returns `None` if the coin isn't a DID or option, so the SDK parses
/// it instead.
fn parse_singleton_spend(
    ctx: &mut SpendContext,
    coin_spend: &CoinSpend,
    delegated_puzzle_hash: &[u8],
    launcher_id: Bytes32,
) -> Result<Option<VaultSingletonSpend>, WalletError> {
    let puzzle = coin_spend.puzzle_reveal.to_clvm(ctx)?;
    let puzzle = Puzzle::parse(ctx, puzzle);
    let solution = coin_spend.solution.to_clvm(ctx)?;

    let (kind, singleton_launcher_id, p2_puzzle, p2_solution) =
        if let Some((did, p2_puzzle)) = DidInfo::parse(ctx, puzzle)? {
            let solution = SingletonSolution::<DidSolution<NodePtr>>::from_clvm(ctx, solution)?;

            let DidSolution::Spend(p2_solution) = solution.inner_solution else {
                return Err(DriverError::InvalidLinkedCustody.into());
            };

            (
                VaultSingletonKind::Did,
                did.launcher_id,
                p2_puzzle,
                p2_solution,
            )
        } else if let Some((option, p2_puzzle, p2_solution)) =
            OptionContract::parse(ctx, coin_spend.coin, puzzle, solution)?
        {
            (
                VaultSingletonKind::Option,
                option.info.launcher_id,
                p2_puzzle,
                p2_solution,
            )
        } else {
            return Ok(None);
        };

    // The singleton must be locked to the vault's own address, so the vault's message is what
    // authorizes its spend.
    let p2_program = Program::from_clvm(ctx, p2_puzzle.ptr())?;

    if parse_p2_singleton(&p2_program).is_none_or(|p2| p2.launcher_id != launcher_id) {
        return Err(DriverError::InvalidLinkedCustody.into());
    }

    let feeder =
        DelegatedPuzzleFeederSolution::<NodePtr, NodePtr, NodePtr>::from_clvm(ctx, p2_solution)?;

    if tree_hash(ctx, feeder.delegated_puzzle).to_bytes() != delegated_puzzle_hash {
        return Err(DriverError::InvalidLinkedCustody.into());
    }

    // The child is parsed like sync parses it, so that its p2 puzzle hash is known rather than
    // taken from what its spend claims.
    let coin = coin_spend.coin;

    let child_p2_puzzle_hash = match kind {
        VaultSingletonKind::Did => {
            let output = ctx.run(puzzle.ptr(), solution)?;

            let child = ctx
                .extract::<Vec<Condition>>(output)?
                .into_iter()
                .find_map(|condition| match condition {
                    Condition::CreateCoin(create_coin) if create_coin.amount % 2 == 1 => Some(
                        Coin::new(coin.coin_id(), create_coin.puzzle_hash, create_coin.amount),
                    ),
                    _ => None,
                });

            match child {
                Some(child) => Some(
                    Did::parse_child(ctx, coin, puzzle, solution, child)?
                        .ok_or(DriverError::InvalidLinkedCustody)?
                        .info
                        .p2_puzzle_hash,
                ),
                None => None,
            }
        }
        VaultSingletonKind::Option => OptionContract::parse_child(ctx, coin, puzzle, solution)?
            .map(|option| option.info.p2_puzzle_hash),
    };

    Ok(Some(VaultSingletonSpend {
        kind,
        launcher_id: singleton_launcher_id,
        coin,
        child_p2_puzzle_hash,
    }))
}

/// The XCH that NFT launchers descended from the minters take from the transaction, which is how
/// NFTs are minted from a singleton. A launcher is created by the minter, or by a coin of no value
/// that the minter creates, and launches a singleton of a mojo. Its value, or the singleton's mojo
/// if it has none, comes from the transaction's XCH. Launchers are distinguished by their amount
/// when a singleton creates more than one, so only small amounts are counted.
fn launcher_amount(
    ctx: &mut SpendContext,
    coin_spends: &[CoinSpend],
    minters: &[Bytes32],
) -> Result<u128, WalletError> {
    let coins: HashMap<Bytes32, Coin> = coin_spends
        .iter()
        .map(|coin_spend| (coin_spend.coin.coin_id(), coin_spend.coin))
        .collect();

    // The coins between the minter and the launcher can't have any value, so the launcher's mojo
    // can only have come from the transaction.
    let is_minted = |launcher: &Coin| {
        let mut parent_id = launcher.parent_coin_info;

        for _ in 0..3 {
            if minters.contains(&parent_id) {
                return true;
            }

            match coins.get(&parent_id) {
                Some(parent) if parent.amount == 0 => parent_id = parent.parent_coin_info,
                _ => return false,
            }
        }

        false
    };

    let mut amount = 0;

    for coin_spend in coin_spends {
        let launcher = coin_spend.coin;

        if launcher.puzzle_hash != SINGLETON_LAUNCHER_HASH.into()
            || launcher.amount > MAX_MINTED_LAUNCHER_AMOUNT
            || !is_minted(&launcher)
        {
            continue;
        }

        let puzzle = coin_spend.puzzle_reveal.to_clvm(ctx)?;
        let solution = coin_spend.solution.to_clvm(ctx)?;
        let output = ctx.run(puzzle, solution)?;

        let launches_mojo = ctx.extract::<Vec<Condition>>(output)?.iter().any(|condition| {
            matches!(condition, Condition::CreateCoin(create_coin) if create_coin.amount == 1)
        });

        if launches_mojo {
            amount += u128::from(launcher.amount.max(1));
        }
    }

    Ok(amount)
}
