use std::{
    collections::{HashMap, HashSet},
    time::{SystemTime, UNIX_EPOCH},
};

use chia_wallet_sdk::{
    chia::puzzle_types::offer::SettlementPaymentsSolution,
    driver::{P2DelegatedConditionsLayer, SpendKind, SpendableAsset},
    prelude::*,
    puzzles::SETTLEMENT_PAYMENT_HASH,
    types::puzzles::P2DelegatedConditionsSolution,
};
use indexmap::IndexMap;
use sage_database::{AssetKind, CoinKind, Database, DeserializePrimitive, P2Puzzle};

mod cats;
mod coin_management;
mod derivations;
mod dids;
mod memos;
mod multi_send;
mod nfts;
mod offer;
mod options;
mod signing;
mod xch;

pub use memos::*;
pub use multi_send::*;
pub use nfts::*;
pub use offer::*;
pub use options::*;

use crate::WalletError;

#[derive(Debug)]
pub struct Wallet {
    pub db: Database,
    pub fingerprint: u32,
    pub intermediate_pk: PublicKey,
    pub genesis_challenge: Bytes32,
    pub agg_sig_constants: AggSigConstants,
    pub change_p2_puzzle_hash: Option<Bytes32>,
}

impl Wallet {
    pub fn new(
        db: Database,
        fingerprint: u32,
        intermediate_pk: PublicKey,
        genesis_challenge: Bytes32,
        agg_sig_constants: AggSigConstants,
        change_p2_puzzle_hash: Option<Bytes32>,
    ) -> Self {
        Self {
            db,
            fingerprint,
            intermediate_pk,
            genesis_challenge,
            agg_sig_constants,
            change_p2_puzzle_hash,
        }
    }

    async fn select_xch_coins(
        &self,
        amount: u64,
        selected_coin_ids: &HashSet<Bytes32>,
    ) -> Result<Vec<Coin>, WalletError> {
        let mut spendable_coins = self.db.selectable_xch_coins().await?;
        spendable_coins.retain(|coin| !selected_coin_ids.contains(&coin.coin_id()));

        Ok(select_coins(spendable_coins, amount)?)
    }

    async fn select_cat_coins(
        &self,
        asset_id: Bytes32,
        amount: u64,
        selected_coin_ids: &HashSet<Bytes32>,
    ) -> Result<Vec<Cat>, WalletError> {
        let mut cat_coins = self.db.selectable_cat_coins(asset_id).await?;
        cat_coins.retain(|cat| !selected_coin_ids.contains(&cat.coin.coin_id()));

        let mut cats = HashMap::new();
        let mut spendable_coins = Vec::new();

        for cat in cat_coins {
            cats.insert(cat.coin, cat);
            spendable_coins.push(cat.coin);
        }

        Ok(select_coins(spendable_coins, amount)?
            .into_iter()
            .map(|coin| cats[&coin])
            .collect())
    }

    pub async fn spend(
        &self,
        ctx: &mut SpendContext,
        selected_coin_ids: Vec<Bytes32>,
        actions: &[Action],
    ) -> Result<Outputs, WalletError> {
        let mut spends = self.prepare_spends(ctx, selected_coin_ids, actions).await?;
        let deltas = spends.apply(ctx, actions)?;
        self.complete_spends(ctx, &deltas, spends).await
    }

    pub async fn prepare_spends_for_selection(
        &self,
        ctx: &mut SpendContext,
        selected_coin_ids: &[Bytes32],
    ) -> Result<Spends, WalletError> {
        let change_p2_puzzle_hash = self.change_p2_puzzle_hash().await?;

        let mut spends = Spends::new(change_p2_puzzle_hash);

        for &coin_id in selected_coin_ids {
            match self.db.coin_kind(coin_id).await? {
                Some(CoinKind::Xch) => {
                    let coin = self
                        .db
                        .xch_coin(coin_id)
                        .await?
                        .ok_or(WalletError::MissingXchCoin(coin_id))?;

                    spends.add(coin);
                }
                Some(CoinKind::Cat) => {
                    let cat = self
                        .db
                        .cat_coin(coin_id)
                        .await?
                        .ok_or(WalletError::MissingCatCoin(coin_id))?;

                    spends.add(cat);
                }
                Some(CoinKind::Did) => {
                    let did = self
                        .db
                        .did_coin(coin_id)
                        .await?
                        .ok_or(WalletError::MissingDidCoin(coin_id))?;

                    spends.add(did.deserialize(ctx)?);
                }
                Some(CoinKind::Nft) => {
                    let nft = self
                        .db
                        .nft_coin(coin_id)
                        .await?
                        .ok_or(WalletError::MissingNftCoin(coin_id))?;

                    spends.add(nft.deserialize(ctx)?);
                }
                Some(CoinKind::Option) => {
                    let option = self
                        .db
                        .option_coin(coin_id)
                        .await?
                        .ok_or(WalletError::MissingOptionCoin(coin_id))?;

                    spends.add(option);
                }
                None => return Err(WalletError::MissingCoin(coin_id)),
            }
        }

        Ok(spends)
    }

    pub async fn prepare_spends(
        &self,
        ctx: &mut SpendContext,
        selected_coin_ids: Vec<Bytes32>,
        actions: &[Action],
    ) -> Result<Spends, WalletError> {
        let mut spends = self
            .prepare_spends_for_selection(ctx, &selected_coin_ids)
            .await?;

        self.select_spends(ctx, &mut spends, actions).await?;

        Ok(spends)
    }

    pub async fn select_spends(
        &self,
        ctx: &mut SpendContext,
        spends: &mut Spends,
        actions: &[Action],
    ) -> Result<(), WalletError> {
        let mut deltas = Deltas::from_actions(actions);

        deltas.update(Id::Xch).input += spends.xch.selected_amount();

        for (&id, cat) in &spends.cats {
            deltas.update(id).input += cat.selected_amount();
        }

        let selected_coin_ids: HashSet<Bytes32> =
            spends.non_settlement_coin_ids().into_iter().collect();

        for &id in deltas.ids() {
            let delta = deltas.get(&id).copied().unwrap_or_default();
            let required_amount = delta.output.saturating_sub(delta.input);

            match id {
                Id::New(_) => {}
                Id::Xch => {
                    if required_amount == 0 && !deltas.is_needed(&id) {
                        continue;
                    }

                    let coins = self
                        .select_xch_coins(required_amount, &selected_coin_ids)
                        .await?;

                    for coin in coins {
                        spends.add(coin);
                    }
                }
                Id::Existing(asset_id) => match self.db.asset_kind(asset_id).await? {
                    Some(AssetKind::Token) => {
                        if required_amount == 0 && !deltas.is_needed(&id) {
                            continue;
                        }

                        let coins = self
                            .select_cat_coins(asset_id, required_amount, &selected_coin_ids)
                            .await?;

                        for coin in coins {
                            spends.add(coin);
                        }
                    }
                    Some(AssetKind::Did) => {
                        if required_amount == 0 && !deltas.is_needed(&id) && delta.output == 0 {
                            continue;
                        }

                        let did = self
                            .db
                            .did(asset_id)
                            .await?
                            .ok_or(WalletError::MissingDid(asset_id))?;

                        spends.add(did.deserialize(ctx)?);
                    }
                    Some(AssetKind::Nft) => {
                        if required_amount == 0 && !deltas.is_needed(&id) && delta.output == 0 {
                            continue;
                        }

                        let nft = self
                            .db
                            .nft(asset_id)
                            .await?
                            .ok_or(WalletError::MissingNft(asset_id))?;

                        spends.add(nft.deserialize(ctx)?);
                    }
                    Some(AssetKind::Option) => {
                        if required_amount == 0 && !deltas.is_needed(&id) && delta.output == 0 {
                            continue;
                        }

                        let option = self
                            .db
                            .option(asset_id)
                            .await?
                            .ok_or(WalletError::MissingOption(asset_id))?;

                        spends.add(option);
                    }
                    None => {
                        if required_amount == 0 && !deltas.is_needed(&id) {
                            continue;
                        }

                        return Err(WalletError::MissingAsset(asset_id));
                    }
                },
            }
        }

        Ok(())
    }

    pub async fn complete_spends(
        &self,
        ctx: &mut SpendContext,
        deltas: &Deltas,
        spends: Spends,
    ) -> Result<Outputs, WalletError> {
        let mut p2_puzzles = IndexMap::new();

        for p2_puzzle_hash in spends.p2_puzzle_hashes() {
            if p2_puzzle_hash == SETTLEMENT_PAYMENT_HASH.into() {
                continue;
            }

            let p2_puzzle = self.db.p2_puzzle(p2_puzzle_hash).await?;

            p2_puzzles.insert(p2_puzzle_hash, p2_puzzle);
        }

        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

        let spends = spends.prepare(ctx, deltas, Relation::AssertConcurrent)?;
        let mut coin_spends = HashMap::new();

        for (asset, kind) in spends.unspent() {
            let spend = match kind {
                SpendKind::Conditions(spend) => {
                    let Some(p2_puzzle) = p2_puzzles.get(&asset.p2_puzzle_hash()) else {
                        return Err(DriverError::MissingKey.into());
                    };

                    match p2_puzzle {
                        P2Puzzle::PublicKey(public_key) => StandardLayer::new(*public_key)
                            .spend_with_conditions(ctx, spend.finish())?,
                        P2Puzzle::Clawback(clawback) => {
                            let Some(public_key) = clawback.public_key else {
                                return Err(DriverError::MissingKey.into());
                            };

                            let custody = StandardLayer::new(public_key);
                            let spend = custody.spend_with_conditions(ctx, spend.finish())?;

                            let clawback = ClawbackV2::new(
                                clawback.sender_puzzle_hash,
                                clawback.receiver_puzzle_hash,
                                clawback.seconds,
                                asset.coin().amount,
                                !matches!(asset, SpendableAsset::Xch(..)),
                            );

                            let is_receiver =
                                custody.tree_hash() == clawback.receiver_puzzle_hash.into();
                            let is_sender =
                                custody.tree_hash() == clawback.sender_puzzle_hash.into();

                            if is_sender && timestamp < clawback.seconds {
                                clawback.sender_spend(ctx, spend)?
                            } else if is_receiver && timestamp >= clawback.seconds {
                                clawback.receiver_spend(ctx, spend)?
                            } else if is_sender || is_receiver {
                                return Err(DriverError::Custom(
                                    "Cannot fulfill clawback spend".to_string(),
                                )
                                .into());
                            } else {
                                return Err(DriverError::MissingKey.into());
                            }
                        }
                        P2Puzzle::Option(underlying) => {
                            let custody = StandardLayer::new(underlying.public_key);
                            let spend = custody.spend_with_conditions(ctx, spend.finish())?;

                            let underlying = OptionUnderlying::new(
                                underlying.launcher_id,
                                custody.tree_hash().into(),
                                underlying.seconds,
                                underlying.amount,
                                underlying.strike_type,
                            );

                            if asset.p2_puzzle_hash() != underlying.tree_hash().into() {
                                return Err(DriverError::MissingKey.into());
                            }

                            underlying.clawback_spend(ctx, spend)?
                        }
                        P2Puzzle::Arbor(key) => P2DelegatedConditionsLayer::new(*key)
                            .construct_spend(
                                ctx,
                                P2DelegatedConditionsSolution::new(
                                    spend.finish().into_iter().collect(),
                                ),
                            )?,
                    }
                }
                SpendKind::Settlement(spend) => SettlementLayer
                    .construct_spend(ctx, SettlementPaymentsSolution::new(spend.finish()))?,
            };

            coin_spends.insert(asset.coin().coin_id(), spend);
        }

        Ok(spends.spend(ctx, coin_spends)?)
    }
}
