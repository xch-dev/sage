//! Clear signing for vault transactions, which checks what a vault spend does before its custody
//! signs it, using the SDK's [`parse_vault_transaction`].

use chia_wallet_sdk::{
    clvm_utils::tree_hash,
    driver::{ClearSigningAsset, Layer, Puzzle, Reveals, SingletonLayer, parse_vault_transaction},
    prelude::*,
    types::puzzles::DelegatedPuzzleFeederSolution,
};

use crate::WalletError;

/// What a vault spend in a transaction does, from the vault's point of view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
}

impl VaultSpendSummary {
    /// Whether the spend recreates the vault with a different custody, or melts it.
    pub fn changes_custody(&self) -> bool {
        self.child_custody_hash != Some(self.custody_hash)
    }
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

    let mut reveals = Reveals::from_coin_spends(&mut ctx, coin_spends)?;
    reveals.reveal_signing_vault(custody_hash, Some(coin));

    let transaction = parse_vault_transaction(reveals, &mut ctx, launcher_id, delegated_spend)?;

    let unaccounted_xch = transaction
        .asset_flows
        .iter()
        .filter(|flow| flow.asset == ClearSigningAsset::Xch)
        .map(|flow| flow.unaccounted_amount)
        .sum();

    Ok(Some(VaultSpendSummary {
        launcher_id,
        coin,
        custody_hash,
        child_custody_hash: transaction.vault_child.map(|child| child.custody_hash),
        reserved_fee: transaction.reserved_fee,
        unaccounted_xch,
    }))
}
