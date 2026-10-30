//! Finding a vault on chain, to import it.
//!
//! A vault can't be found from a key, since keys only appear behind hashes or inside memos, so it's
//! found from its launcher id, or from an address it has spent from, whose puzzle reveal names the
//! vault that owns it.

use chia_wallet_sdk::{
    driver::P2Singleton,
    prelude::*,
    types::puzzles::{
        DelegatedPuzzleFeederArgs, INDEX_WRAPPER_HASH, IndexWrapperArgs, SingletonMember,
    },
};

use crate::{WalletError, WalletPeer};

/// How many spent coins at an address are checked for the vault that owns it.
const MAX_ADDRESS_SPENDS_CHECKED: usize = 10;

/// Follows a vault from its launcher to its latest coin. Returns `None` if the launcher doesn't
/// exist or hasn't created a singleton yet. Once the vault is melted, its latest coin is spent.
pub async fn fetch_latest_vault_coin(
    peer: &WalletPeer,
    launcher_id: Bytes32,
    genesis_challenge: Bytes32,
) -> Result<Option<CoinState>, WalletError> {
    Ok(fetch_vault_lineage(peer, launcher_id, genesis_challenge)
        .await?
        .and_then(|lineage| lineage.last().copied()))
}

/// Follows a vault from its launcher, returning each of its singleton coins in order, from the eve
/// coin to the latest coin. Returns `None` if the launcher doesn't exist or hasn't created a
/// singleton yet.
pub async fn fetch_vault_lineage(
    peer: &WalletPeer,
    launcher_id: Bytes32,
    genesis_challenge: Bytes32,
) -> Result<Option<Vec<CoinState>>, WalletError> {
    let Some(launcher) = peer
        .fetch_optional_coin(launcher_id, genesis_challenge)
        .await?
    else {
        return Ok(None);
    };

    if launcher.spent_height.is_none() {
        return Ok(None);
    }

    let mut lineage = Vec::new();
    let mut coin_id = launcher_id;

    while let Some(child) = peer.try_fetch_singleton_child(coin_id).await? {
        lineage.push(child);

        if child.spent_height.is_none() {
            break;
        }

        coin_id = child.coin.coin_id();
    }

    Ok((!lineage.is_empty()).then_some(lineage))
}

/// What was found about the vault that owns an address.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VaultAddressLookup {
    Found(P2Singleton),
    /// No coin has been spent from the address, so its puzzle hasn't been revealed.
    NeverSpent,
    /// Coins have been spent from the address, but not by a vault.
    NotVault,
}

/// Finds the vault that owns an address, from the puzzle reveal of a coin spent from it.
pub async fn find_vault_by_address(
    peer: &WalletPeer,
    p2_puzzle_hash: Bytes32,
    genesis_challenge: Bytes32,
) -> Result<VaultAddressLookup, WalletError> {
    let spent = peer
        .fetch_spent_coins_by_puzzle_hash(p2_puzzle_hash, genesis_challenge)
        .await?;

    if spent.is_empty() {
        return Ok(VaultAddressLookup::NeverSpent);
    }

    for coin_state in spent.into_iter().take(MAX_ADDRESS_SPENDS_CHECKED) {
        let Some(spent_height) = coin_state.spent_height else {
            continue;
        };

        let (puzzle_reveal, _solution) = peer
            .fetch_puzzle_solution(coin_state.coin.coin_id(), spent_height)
            .await?;

        if let Some(p2_singleton) = parse_p2_singleton(&puzzle_reveal)
            && p2_singleton.tree_hash() == p2_puzzle_hash.into()
        {
            return Ok(VaultAddressLookup::Found(p2_singleton));
        }
    }

    Ok(VaultAddressLookup::NotVault)
}

/// Parses a P2 singleton puzzle, which is a top level MIPS tree with a single singleton member.
pub fn parse_p2_singleton(puzzle: &Program) -> Option<P2Singleton> {
    let mut allocator = Allocator::new();
    let puzzle = puzzle.to_clvm(&mut allocator).ok()?;
    let puzzle = Puzzle::parse(&allocator, puzzle);

    let curried = puzzle.as_curried()?;

    if curried.mod_hash != INDEX_WRAPPER_HASH {
        return None;
    }

    let index = IndexWrapperArgs::<usize, NodePtr>::from_clvm(&allocator, curried.args).ok()?;
    let feeder = Puzzle::parse(&allocator, index.inner_puzzle).as_curried()?;
    let feeder = DelegatedPuzzleFeederArgs::<NodePtr>::from_clvm(&allocator, feeder.args).ok()?;
    let member = Puzzle::parse(&allocator, feeder.inner_puzzle).as_curried()?;

    if member.mod_hash != SingletonMember::mod_hash() {
        return None;
    }

    let member = SingletonMember::from_clvm(&allocator, member.args).ok()?;

    let p2_singleton = P2Singleton::new(member.singleton_struct.launcher_id, index.nonce);

    // The puzzle must be exactly the P2 singleton, rather than one with the same members.
    (p2_singleton.tree_hash() == puzzle.curried_puzzle_hash()).then_some(p2_singleton)
}

#[cfg(test)]
mod tests {
    use chia_wallet_sdk::driver::SpendContext;

    use super::*;

    #[test]
    fn test_parse_p2_singleton() -> anyhow::Result<()> {
        let mut ctx = SpendContext::new();

        for nonce in [0, 7] {
            let p2_singleton = P2Singleton::new(Bytes32::new([1; 32]), nonce);
            let delegated_spend = ctx.delegated_spend(Conditions::new())?;
            let spend = p2_singleton.spend(&mut ctx, Bytes32::new([2; 32]), 1, delegated_spend)?;
            let puzzle = ctx.serialize(&spend.puzzle)?;

            assert_eq!(parse_p2_singleton(&puzzle), Some(p2_singleton));
        }

        // Any other puzzle isn't one.
        let member = ctx.curry(SingletonMember::new(Bytes32::new([1; 32])))?;
        let puzzle = ctx.serialize(&member)?;
        assert_eq!(parse_p2_singleton(&puzzle), None);

        Ok(())
    }
}
