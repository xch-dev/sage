use chia_wallet_sdk::{
    chia::puzzle_types::{cat::CatSolution, singleton::SingletonStruct},
    driver::{Delta, IntermediateLauncher, NftMint},
    prelude::*,
    puzzles::NFT_METADATA_UPDATER_DEFAULT_HASH,
    types::{
        MessageFlags, MessageSide,
        puzzles::{EverythingWithSingletonTailArgs, EverythingWithSingletonTailSolution},
    },
};
use sage_database::{SerializePrimitive, SerializedNft};

use crate::{WalletError, WalletNftMint};

use super::{SpendableVault, Wallet};

/// How many nonces are tried to find the TAIL of a CAT that a vault can issue more of, or an
/// unused one for a new CAT.
const MAX_VAULT_TAIL_NONCES: u32 = 1000;

/// The TAIL of a CAT that the vault can issue more of, and melt, by sending it a message.
pub fn vault_tail(launcher_id: Bytes32, nonce: u32) -> EverythingWithSingletonTailArgs {
    EverythingWithSingletonTailArgs::new(launcher_id, nonce.to_be_bytes().to_vec().into())
}

impl Wallet {
    /// Issues a CAT whose TAIL lets this vault issue more of it later, as Cloud Wallet does. The
    /// TAIL is unique to the vault and the first unused nonce.
    pub async fn issue_vault_cat(
        &self,
        amount: u64,
        fee: u64,
    ) -> Result<(Vec<CoinSpend>, Bytes32), WalletError> {
        let launcher_id = self
            .vault_launcher_id()
            .ok_or(WalletError::VaultSpendNotSupported)?;

        let mut nonce = None;

        for candidate in 0..MAX_VAULT_TAIL_NONCES {
            let asset_id = vault_tail(launcher_id, candidate).curry_tree_hash().into();

            if self.db.asset(asset_id).await?.is_none() {
                nonce = Some(candidate);
                break;
            }
        }

        let nonce = nonce.ok_or(WalletError::VaultTailNoncesExhausted)?;
        let vault = self.spendable_vault().await?;

        let mut ctx = SpendContext::new();
        let tail_spend = vault_tail_spend(&mut ctx, &vault, nonce)?;

        let actions = [
            Action::fee(fee),
            Action::issue_cat(tail_spend, None, amount),
        ];
        let outputs = self.spend(&mut ctx, vec![], &actions).await?;

        Ok((ctx.take(), outputs.cats[&Id::New(1)][0].info.asset_id))
    }

    /// Issues more of a CAT whose TAIL is controlled by this vault.
    pub async fn issue_more_vault_cat(
        &self,
        asset_id: Bytes32,
        amount: u64,
        fee: u64,
    ) -> Result<Vec<CoinSpend>, WalletError> {
        let launcher_id = self
            .vault_launcher_id()
            .ok_or(WalletError::VaultSpendNotSupported)?;

        let nonce = (0..MAX_VAULT_TAIL_NONCES)
            .find(|&nonce| {
                Bytes32::from(vault_tail(launcher_id, nonce).curry_tree_hash()) == asset_id
            })
            .ok_or(WalletError::NotVaultTail(asset_id))?;

        let vault = self.spendable_vault().await?;

        let mut ctx = SpendContext::new();
        let tail_spend = vault_tail_spend(&mut ctx, &vault, nonce)?;

        let actions = [
            Action::fee(fee),
            Action::run_tail(Id::Existing(asset_id), tail_spend, Delta::new(amount, 0)),
        ];
        self.spend(&mut ctx, vec![], &actions).await?;

        Ok(ctx.take())
    }

    /// Mints NFTs with the vault as their minter. The vault creates each NFT's intermediate
    /// launcher itself, so the minter is found by looking up the launcher's ancestry, as for a
    /// DID. The NFTs aren't assigned to a DID.
    pub async fn mint_vault_nfts(
        &self,
        fee: u64,
        mints: Vec<WalletNftMint>,
    ) -> Result<(Vec<CoinSpend>, Vec<SerializedNft>), WalletError> {
        let vault = self.spendable_vault().await?;
        let default_p2_puzzle_hash = self.change_p2_puzzle_hash().await?;

        let mut ctx = SpendContext::new();
        let mut vault_conditions = Conditions::new();
        let mut nfts = Vec::new();

        let total = mints.len();

        for (index, mint) in mints.into_iter().enumerate() {
            let launcher = IntermediateLauncher::new(vault.vault.coin.coin_id(), index, total)
                .create(&mut ctx)?;

            let metadata = ctx.alloc_hashed(&mint.metadata)?;

            let (conditions, nft) = launcher.mint_nft(
                &mut ctx,
                &NftMint {
                    metadata,
                    metadata_updater_puzzle_hash: NFT_METADATA_UPDATER_DEFAULT_HASH.into(),
                    royalty_puzzle_hash: mint.royalty_puzzle_hash.unwrap_or(default_p2_puzzle_hash),
                    royalty_basis_points: mint.royalty_basis_points,
                    p2_puzzle_hash: mint.p2_puzzle_hash.unwrap_or(default_p2_puzzle_hash),
                    transfer_condition: None,
                },
            )?;

            vault_conditions = vault_conditions.extend(conditions);
            nfts.push(nft.serialize(&ctx)?);
        }

        // Each launcher is a mojo, which the intermediate coins don't have, so it's paid for by
        // leaving that much of the vault's XCH unspent.
        let actions = [Action::fee(fee)];
        let mut deltas = Deltas::from_actions(&actions);
        *deltas.update(Id::Xch) += Delta::new(0, total as u64);
        deltas.set_needed(Id::Xch);

        let mut spends = self.prepare_spends_for_selection(&mut ctx, &[]).await?;
        self.select_spends_for_deltas(&mut ctx, &mut spends, deltas.clone(), &[])
            .await?;
        spends.apply(&mut ctx, &actions)?;

        self.complete_spends_with_vault_conditions(&mut ctx, &deltas, spends, vault_conditions)
            .await?;

        Ok((ctx.take(), nfts))
    }
}

/// The spend of a vault's TAIL, which receives the vault's message with the CAT's delta.
fn vault_tail_spend(
    ctx: &mut SpendContext,
    vault: &SpendableVault,
    nonce: u32,
) -> Result<Spend, WalletError> {
    let tail = ctx.curry(vault_tail(vault.vault.info.launcher_id, nonce))?;
    let solution = ctx.alloc(&EverythingWithSingletonTailSolution::new(
        vault.vault.info.custody_hash.into(),
    ))?;
    Ok(Spend::new(tail, solution))
}

/// The messages that the vault sends to the CAT spends which run its TAIL, whose content is the
/// spend's extra delta, as the SDK's `vault_tail_messages` does.
pub(super) fn vault_tail_messages(
    ctx: &mut SpendContext,
    launcher_id: Bytes32,
    coin_spends: &[CoinSpend],
) -> Result<Vec<SendMessage<NodePtr>>, WalletError> {
    let expected_struct_hash: Bytes32 = SingletonStruct::new(launcher_id).tree_hash().into();

    let mut messages = Vec::new();

    for coin_spend in coin_spends {
        let puzzle = coin_spend.puzzle_reveal.to_clvm(ctx)?;
        let puzzle = Puzzle::parse(ctx, puzzle);
        let solution = coin_spend.solution.to_clvm(ctx)?;

        let Some(parsed) = Cat::parse(ctx, coin_spend.coin, puzzle, solution)? else {
            continue;
        };

        let output = ctx.run(parsed.p2_puzzle.ptr(), parsed.p2_solution)?;
        let conditions: Vec<Condition> = ctx.extract(output)?;

        let Some(run_cat_tail) = conditions.iter().find_map(Condition::as_run_cat_tail) else {
            continue;
        };

        let Some(curried) = CurriedPuzzle::parse(ctx, run_cat_tail.program) else {
            continue;
        };

        if curried.mod_hash != EverythingWithSingletonTailArgs::mod_hash() {
            continue;
        }

        let args = ctx.extract::<EverythingWithSingletonTailArgs>(curried.args)?;

        if args.singleton_struct_hash != expected_struct_hash {
            continue;
        }

        let cat_solution = ctx.extract::<CatSolution<NodePtr>>(solution)?;
        let extra_delta = ctx.alloc(&cat_solution.extra_delta)?;
        let extra_delta: Bytes = ctx.atom(extra_delta).as_ref().to_vec().into();

        let mode = MessageFlags::PUZZLE.encode(MessageSide::Sender)
            | MessageFlags::COIN.encode(MessageSide::Receiver);
        let coin_id = ctx.alloc(&coin_spend.coin.coin_id())?;

        messages.push(SendMessage::new(mode, extra_delta, vec![coin_id]));
    }

    Ok(messages)
}
