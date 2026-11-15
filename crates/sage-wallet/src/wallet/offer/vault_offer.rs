use std::collections::HashMap;

use chia_wallet_sdk::{
    chia::puzzle_types::{LineageProof, offer::SettlementPaymentsSolution},
    clvm_traits::{clvm_list, clvm_quote},
    clvm_utils::TreeHash,
    driver::{
        P2ConditionsOrSingleton, P2ConditionsOrSingletonVersion, SpendKind, SpendableAsset,
        coin_amount,
    },
    prelude::*,
};
use indexmap::IndexMap;
use rand::random;

use crate::{Offered, PreSplitOffer, Requested, Wallet, WalletError, sign_with_secret_keys};

use super::{offer_actions, offer_requested_payments};

/// The coin locked to an offer's fixed conditions, which the vault pays an asset into before the
/// offer can be taken.
#[derive(Debug, Clone)]
struct PreSplitCoin {
    p2: P2ConditionsOrSingleton,
    conditions: Conditions,
    quoted_conditions_hash: TreeHash,
    amount: u64,
}

impl Wallet {
    /// Makes an offer from a vault, which pays each offered asset into a coin locked to the
    /// offer's fixed conditions, as Cloud Wallet does. Anyone can spend those coins with their
    /// fixed conditions, so the offer spends them without the vault's signature, and the vault
    /// stays usable while the offer is open. The vault can take them back to cancel the offer.
    ///
    /// Each coin's fixed conditions pay its asset into settlement, assert the requested payments,
    /// and require a signature from a random key that's discarded once the offer is signed. The
    /// vault's transaction reveals the fixed conditions in the coins' memos, so that the vault can
    /// find the coins again. Without the signature, anyone could take the offer from what's on
    /// chain, rather than only those it was shared with.
    ///
    /// Only XCH and CATs can be offered, since singletons can't be locked to fixed conditions yet.
    pub(super) async fn make_vault_offer(
        &self,
        offered: Offered,
        requested: Requested,
        expires_at: Option<u64>,
        split_fee: u64,
    ) -> Result<PreSplitOffer, WalletError> {
        if !offered.nfts.is_empty() || !offered.options.is_empty() {
            return Err(WalletError::VaultOfferAssetNotSupported);
        }

        let launcher_id = self
            .vault_launcher_id()
            .ok_or(WalletError::VaultSpendNotSupported)?;
        let vault = self.spendable_vault().await?;
        let vault_puzzle_hash = self.change_p2_puzzle_hash().await?;

        let mut ctx = SpendContext::new();
        let mut asset_info = AssetInfo::new();

        let actions = offer_actions(&mut ctx, &offered, &requested)?;
        let deltas = Deltas::from_actions(&actions);

        // The vault coin is spent by the transaction that creates the pre-split coins, so it makes
        // the requested payments unique to this offer.
        let nonce = Offer::nonce(vec![vault.vault.coin.coin_id()]);
        let p2_puzzle_hash = offered.p2_puzzle_hash.unwrap_or(vault_puzzle_hash);
        let requested_payments =
            offer_requested_payments(&mut ctx, requested, nonce, p2_puzzle_hash, &mut asset_info)?;

        let offer_key = SecretKey::from_seed(&random::<[u8; 32]>());

        // Every pre-split coin asserts all of the requested payments, so that none of them can be
        // taken on its own without making them.
        let mut common = Conditions::new()
            .extend(requested_payments.assertions(&mut ctx, &asset_info)?)
            .agg_sig_me(offer_key.public_key(), Bytes::default());

        if let Some(expires_at) = expires_at {
            common = common.assert_before_seconds_absolute(expires_at);
        }

        // The XCH coin also reserves the fee, so it's needed even if no XCH is offered.
        let mut amounts = Vec::new();

        let xch_amount = coin_amount(deltas.get(&Id::Xch).map_or(0, |delta| delta.output))?;

        if xch_amount > 0 {
            amounts.push((Id::Xch, xch_amount));
        }

        for &asset_id in offered.cats.keys() {
            let id = Id::Existing(asset_id);
            amounts.push((id, coin_amount(deltas.get(&id).map_or(0, |d| d.output))?));
        }

        // The fixed conditions don't depend on the coins' ids, so they're found from stand-in
        // coins first, which gives the puzzle hashes to pay the assets into.
        let mut stand_ins = Spends::new(vault_puzzle_hash);

        for &(id, amount) in &amounts {
            self.add_pre_split_coin(&mut stand_ins, id, Bytes32::default(), amount)
                .await?;
        }

        let fixed = pre_split_conditions(&mut ctx, stand_ins, &actions, &common)?;

        let mut pre_split_coins = IndexMap::new();

        for (id, conditions) in fixed {
            let quoted = ctx.alloc(&clvm_quote!(&conditions))?;
            let quoted_conditions_hash = ctx.tree_hash(quoted);

            let p2 = P2ConditionsOrSingleton::from_quoted_conditions_hash(
                launcher_id,
                0,
                quoted_conditions_hash.into(),
                P2ConditionsOrSingletonVersion::FixedPuzzleMember,
            );

            let amount = amounts
                .iter()
                .find(|(amount_id, _)| *amount_id == id)
                .map_or(0, |(_, amount)| *amount);

            pre_split_coins.insert(
                id,
                PreSplitCoin {
                    p2,
                    conditions,
                    quoted_conditions_hash,
                    amount,
                },
            );
        }

        // The vault pays each asset into its pre-split coin, whose memos reveal the vault's address
        // and the fixed conditions.
        let mut split_actions = vec![Action::fee(split_fee)];

        for (&id, coin) in &pre_split_coins {
            let memos = ctx.memos(&clvm_list!(vault_puzzle_hash, &coin.conditions))?;
            split_actions.push(Action::send(
                id,
                coin.p2.tree_hash().into(),
                coin.amount,
                memos,
            ));
        }

        let mut split_spends = self
            .prepare_spends(&mut ctx, offered.selected_coin_ids.clone(), &split_actions)
            .await?;
        let split_deltas = split_spends.apply(&mut ctx, &split_actions)?;
        let outputs = self
            .complete_spends(&mut ctx, &split_deltas, split_spends)
            .await?;
        let split_coin_spends = ctx.take();

        // The offer spends the coins that the vault's transaction creates.
        let mut spends = Spends::new(vault_puzzle_hash);

        for (&id, coin) in &pre_split_coins {
            let puzzle_hash: Bytes32 = coin.p2.tree_hash().into();

            if id == Id::Xch {
                let coin = outputs
                    .xch
                    .iter()
                    .find(|output| {
                        output.puzzle_hash == puzzle_hash && output.amount == coin.amount
                    })
                    .ok_or(WalletError::MissingPreSplitCoin)?;

                spends.add(*coin);
            } else {
                let cat = outputs
                    .cats
                    .get(&id)
                    .and_then(|cats| {
                        cats.iter().find(|cat| {
                            cat.info.p2_puzzle_hash == puzzle_hash && cat.coin.amount == coin.amount
                        })
                    })
                    .ok_or(WalletError::MissingPreSplitCoin)?;

                spends.add(*cat);
            }
        }

        let deltas = spends.apply(&mut ctx, &actions)?;
        let finished = spends.prepare(&mut ctx, &deltas, Relation::None)?;

        let mut coin_spends = HashMap::new();

        for (asset, kind) in finished.unspent() {
            let spend = match kind {
                SpendKind::Conditions(spend) => {
                    let id = asset_id(&asset);
                    let coin = pre_split_coins
                        .get(&id)
                        .ok_or(WalletError::MissingPreSplitCoin)?;

                    let conditions = spend.finish().extend(common.clone());
                    let quoted = ctx.alloc(&clvm_quote!(&conditions))?;

                    // The conditions have to be the ones the coin was locked to.
                    if ctx.tree_hash(quoted) != coin.quoted_conditions_hash {
                        return Err(WalletError::MissingPreSplitCoin);
                    }

                    coin.p2
                        .fixed_spend(&mut ctx, Spend::new(quoted, NodePtr::NIL))?
                }
                SpendKind::Settlement(spend) => SettlementLayer
                    .construct_spend(&mut ctx, SettlementPaymentsSolution::new(spend.finish()))?,
            };

            coin_spends.insert(asset.coin().coin_id(), spend);
        }

        finished.spend(&mut ctx, coin_spends)?;

        let offer_spend_bundle = sign_with_secret_keys(
            SpendBundle::new(ctx.take(), Signature::default()),
            &self.agg_sig_constants,
            &[offer_key],
            true,
        )?;

        let offer = Offer::from_input_spend_bundle(
            &mut ctx,
            offer_spend_bundle,
            requested_payments,
            asset_info,
        )?;

        Ok(PreSplitOffer {
            offer: offer.to_spend_bundle(&mut ctx)?,
            split_coin_spends: Some(split_coin_spends),
        })
    }

    /// Adds a stand-in coin of the asset, with the given p2 puzzle hash.
    async fn add_pre_split_coin(
        &self,
        spends: &mut Spends,
        id: Id,
        p2_puzzle_hash: Bytes32,
        amount: u64,
    ) -> Result<(), WalletError> {
        match id {
            Id::Xch => spends.add(Coin::new(Bytes32::default(), p2_puzzle_hash, amount)),
            Id::Existing(asset_id) => {
                let hidden_puzzle_hash = self
                    .db
                    .asset(asset_id)
                    .await?
                    .and_then(|asset| asset.hidden_puzzle_hash);

                let info = CatInfo::new(asset_id, hidden_puzzle_hash, p2_puzzle_hash);

                spends.add(Cat::new(
                    Coin::new(Bytes32::default(), info.puzzle_hash().into(), amount),
                    Some(LineageProof {
                        parent_parent_coin_info: Bytes32::default(),
                        parent_inner_puzzle_hash: Bytes32::default(),
                        parent_amount: 0,
                    }),
                    info,
                ));
            }
            Id::New(_) => return Err(WalletError::MissingPreSplitCoin),
        }

        Ok(())
    }
}

/// The fixed conditions of each pre-split coin in the spends, which are what the actions have it
/// output, followed by the conditions that every pre-split coin outputs.
fn pre_split_conditions(
    ctx: &mut SpendContext,
    mut spends: Spends,
    actions: &[Action],
    common: &Conditions,
) -> Result<IndexMap<Id, Conditions>, WalletError> {
    let deltas = spends.apply(ctx, actions)?;
    let finished = spends.prepare(ctx, &deltas, Relation::None)?;

    let mut conditions = IndexMap::new();

    for (asset, kind) in finished.unspent() {
        if let SpendKind::Conditions(spend) = kind {
            conditions.insert(asset_id(&asset), spend.finish().extend(common.clone()));
        }
    }

    Ok(conditions)
}

fn asset_id(asset: &SpendableAsset) -> Id {
    match asset {
        SpendableAsset::Cat(cat) => Id::Existing(cat.info.asset_id),
        _ => Id::Xch,
    }
}
