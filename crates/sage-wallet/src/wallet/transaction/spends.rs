use std::{collections::HashMap, mem};

use chia::protocol::{Bytes, Bytes32, Coin};
use chia_puzzles::SINGLETON_LAUNCHER_HASH;
use chia_wallet_sdk::{
    driver::{Cat, Did, HashedPtr, Launcher, Nft, OptionContract, SpendContext, StandardLayer},
    types::Conditions,
};
use indexmap::{IndexMap, IndexSet};
use itertools::Itertools;

use crate::{wallet::memos::calculate_memos, Wallet, WalletError};

use super::{Action, Id, Selection, Summary, TransactionConfig};

#[derive(Debug, Clone)]
pub struct Spends {
    pub xch: AssetSpends,
    pub cats: IndexMap<Id, AssetSpends>,
    pub dids: IndexMap<Id, AssetSpends>,
    pub nfts: IndexMap<Id, AssetSpends>,
    pub options: IndexMap<Id, AssetSpends>,
}

#[derive(Debug, Clone)]
pub struct AssetSpends {
    pub items: Vec<AssetSpend>,
    pub launcher_index: u64,
    pub launcher_multiplier: u64,
    pub parent_index: usize,
    pub was_created: bool,
}

impl AssetSpends {
    pub fn did(&mut self) -> Result<(&mut AssetSpend, Did<HashedPtr>), WalletError> {
        self.items
            .iter_mut()
            .find_map(|spend| {
                if let AssetCoin::Did(did) = spend.coin {
                    Some((spend, did))
                } else {
                    None
                }
            })
            .ok_or(WalletError::MissingAsset)
    }

    pub fn nft(&mut self) -> Result<(&mut AssetSpend, Nft<HashedPtr>), WalletError> {
        self.items
            .iter_mut()
            .find_map(|spend| {
                if let AssetCoin::Nft(nft) = spend.coin {
                    Some((spend, nft))
                } else {
                    None
                }
            })
            .ok_or(WalletError::MissingAsset)
    }

    pub fn option(&mut self) -> Result<(&mut AssetSpend, OptionContract), WalletError> {
        self.items
            .iter_mut()
            .find_map(|spend| {
                if let AssetCoin::Option(option) = spend.coin {
                    Some((spend, option))
                } else {
                    None
                }
            })
            .ok_or(WalletError::MissingAsset)
    }
}

#[derive(Debug, Clone)]
pub struct AssetSpend {
    pub coin: AssetCoin,
    pub p2: StandardLayer,
    pub payments: IndexSet<Payment>,
    pub conditions: Conditions,
}

impl AssetSpend {
    pub fn new(coin: AssetCoin, p2: StandardLayer) -> Self {
        Self {
            coin,
            p2,
            payments: IndexSet::new(),
            conditions: Conditions::new(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum AssetCoin {
    Xch(Coin),
    Cat(Cat),
    Did(Did<HashedPtr>),
    Nft(Nft<HashedPtr>),
    Option(OptionContract),
}

impl AssetCoin {
    #[must_use]
    pub fn child(&self, p2_puzzle_hash: Bytes32, amount: u64) -> Self {
        match self {
            Self::Xch(coin) => Self::Xch(Coin::new(coin.coin_id(), p2_puzzle_hash, amount)),
            Self::Cat(cat) => Self::Cat(cat.wrapped_child(p2_puzzle_hash, amount)),
            Self::Did(did) => Self::Xch(Coin::new(did.coin.coin_id(), p2_puzzle_hash, amount)),
            Self::Nft(nft) => Self::Xch(Coin::new(nft.coin.coin_id(), p2_puzzle_hash, amount)),
            Self::Option(option) => {
                Self::Xch(Coin::new(option.coin.coin_id(), p2_puzzle_hash, amount))
            }
        }
    }

    pub fn coin(&self) -> Coin {
        match self {
            Self::Xch(coin) => *coin,
            Self::Cat(cat) => cat.coin,
            Self::Did(did) => did.coin,
            Self::Nft(nft) => nft.coin,
            Self::Option(option) => option.coin,
        }
    }

    pub fn p2_puzzle_hash(&self) -> Bytes32 {
        match self {
            Self::Xch(coin) => coin.puzzle_hash,
            Self::Cat(cat) => cat.p2_puzzle_hash,
            Self::Did(did) => did.info.p2_puzzle_hash,
            Self::Nft(nft) => nft.info.p2_puzzle_hash,
            Self::Option(option) => option.info.p2_puzzle_hash,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Payment {
    pub puzzle_hash: Bytes32,
    pub amount: u64,
}

impl Payment {
    pub fn new(puzzle_hash: Bytes32, amount: u64) -> Self {
        Self {
            puzzle_hash,
            amount,
        }
    }
}

impl AssetSpends {
    pub fn new(items: Vec<AssetSpend>, launcher_multiplier: u64, was_created: bool) -> Self {
        Self {
            items,
            launcher_index: 0,
            launcher_multiplier,
            parent_index: 0,
            was_created,
        }
    }

    pub fn make_payment(
        &mut self,
        ctx: &mut SpendContext,
        payment: Payment,
    ) -> Result<&mut AssetSpend, WalletError> {
        // This weird duplicated logic is due to a flaw in the borrow checker.
        if self
            .items
            .iter()
            .any(|item| !item.payments.contains(&payment))
        {
            let item = self
                .items
                .iter_mut()
                .find(|item| !item.payments.contains(&payment))
                .expect("missing item");
            item.payments.insert(payment);
            return Ok(item);
        }

        let Some(parent) = self.items.iter_mut().find(|item| {
            !item
                .payments
                .contains(&Payment::new(item.coin.p2_puzzle_hash(), 0))
        }) else {
            return Err(WalletError::NoIntermediateParent);
        };

        parent
            .payments
            .insert(Payment::new(parent.coin.p2_puzzle_hash(), 0));

        parent.conditions = mem::take(&mut parent.conditions).create_coin(
            parent.coin.p2_puzzle_hash(),
            0,
            calculate_memos(
                ctx,
                parent.coin.p2_puzzle_hash(),
                matches!(parent.coin, AssetCoin::Cat(..)),
                None,
            )?,
        );

        let child = parent.coin.child(parent.coin.p2_puzzle_hash(), 0);
        let p2 = parent.p2;

        self.items.push(AssetSpend::new(child, p2));
        let item = self.items.last_mut().expect("item should exist");

        item.payments.insert(payment);

        Ok(item)
    }

    pub fn create_coin(
        &mut self,
        ctx: &mut SpendContext,
        p2_puzzle_hash: Bytes32,
        amount: u64,
        include_hint: bool,
        memos: Option<Vec<Bytes>>,
    ) -> Result<(), WalletError> {
        let item = self.make_payment(ctx, Payment::new(p2_puzzle_hash, amount))?;

        item.conditions = mem::take(&mut item.conditions).create_coin(
            p2_puzzle_hash,
            amount,
            calculate_memos(ctx, p2_puzzle_hash, include_hint, memos)?,
        );

        Ok(())
    }

    pub fn create_launcher(
        &mut self,
        ctx: &mut SpendContext,
    ) -> Result<(&mut AssetSpend, Launcher), WalletError> {
        let launcher_amount = self.launcher_index * self.launcher_multiplier;
        self.launcher_index += 1;

        let p2_puzzle_hash = SINGLETON_LAUNCHER_HASH.into();

        let item = self.make_payment(ctx, Payment::new(p2_puzzle_hash, launcher_amount))?;

        let launcher =
            Launcher::new(item.coin.coin().coin_id(), launcher_amount).with_singleton_amount(1);

        Ok((item, launcher))
    }

    pub fn create_from_unique_parent(
        &mut self,
        ctx: &mut SpendContext,
    ) -> Result<&mut AssetSpend, WalletError> {
        // This weird duplicated logic is due to a flaw in the borrow checker.
        if self.parent_index < self.items.len() {
            return Ok(self.items.get_mut(self.parent_index).expect("missing item"));
        }

        let Some(parent) = self.items.iter_mut().find(|item| {
            !item
                .payments
                .contains(&Payment::new(item.coin.p2_puzzle_hash(), 0))
        }) else {
            return Err(WalletError::NoIntermediateParent);
        };

        parent
            .payments
            .insert(Payment::new(parent.coin.p2_puzzle_hash(), 0));

        parent.conditions = mem::take(&mut parent.conditions).create_coin(
            parent.coin.p2_puzzle_hash(),
            0,
            calculate_memos(
                ctx,
                parent.coin.p2_puzzle_hash(),
                matches!(parent.coin, AssetCoin::Cat(..)),
                None,
            )?,
        );

        let child = parent.coin.child(parent.coin.p2_puzzle_hash(), 0);
        let p2 = parent.p2;

        self.items.push(AssetSpend::new(child, p2));
        let item = self.items.last_mut().expect("item should exist");

        self.parent_index += 1;

        Ok(item)
    }
}

impl Wallet {
    pub async fn spend(
        &self,
        ctx: &mut SpendContext,
        summary: &Summary,
        selection: &Selection,
        tx: &TransactionConfig,
    ) -> Result<Spends, WalletError> {
        let change_puzzle_hash = self.p2_puzzle_hash(false, true).await?;

        let mut p2 = HashMap::new();

        for p2_puzzle_hash in selection
            .xch
            .coins
            .iter()
            .map(|coin| coin.puzzle_hash)
            .chain(
                selection
                    .cats
                    .values()
                    .flat_map(|selected| selected.coins.iter().map(|cat| cat.p2_puzzle_hash)),
            )
            .chain(selection.nfts.values().map(|nft| nft.info.p2_puzzle_hash))
            .chain(selection.dids.values().map(|did| did.info.p2_puzzle_hash))
            .chain(
                selection
                    .options
                    .values()
                    .map(|option| option.info.p2_puzzle_hash),
            )
        {
            let synthetic_key = self.db.synthetic_key(p2_puzzle_hash).await?;
            p2.insert(p2_puzzle_hash, StandardLayer::new(synthetic_key));
        }

        let xch = AssetSpends::new(
            selection
                .xch
                .coins
                .iter()
                .map(|&coin| AssetSpend::new(AssetCoin::Xch(coin), p2[&coin.puzzle_hash]))
                .collect(),
            1,
            false,
        );

        let cats = selection
            .cats
            .iter()
            .map(|(&id, selected)| {
                let spends = AssetSpends::new(
                    selected
                        .coins
                        .iter()
                        .map(|&cat| AssetSpend::new(AssetCoin::Cat(cat), p2[&cat.p2_puzzle_hash]))
                        .collect(),
                    1,
                    false,
                );
                (id, spends)
            })
            .collect();

        let dids = selection
            .dids
            .iter()
            .map(|(&id, &did)| {
                let spends = AssetSpends::new(
                    vec![AssetSpend::new(
                        AssetCoin::Did(did),
                        p2[&did.info.p2_puzzle_hash],
                    )],
                    2,
                    false,
                );
                (id, spends)
            })
            .collect();

        let nfts = selection
            .nfts
            .iter()
            .map(|(&id, &nft)| {
                let spends = AssetSpends::new(
                    vec![AssetSpend::new(
                        AssetCoin::Nft(nft),
                        p2[&nft.info.p2_puzzle_hash],
                    )],
                    2,
                    false,
                );
                (id, spends)
            })
            .collect();

        let options = selection
            .options
            .iter()
            .map(|(&id, &option)| {
                let spends = AssetSpends::new(
                    vec![AssetSpend::new(
                        AssetCoin::Option(option),
                        p2[&option.info.p2_puzzle_hash],
                    )],
                    2,
                    false,
                );
                (id, spends)
            })
            .collect();

        let mut spends = Spends {
            xch,
            cats,
            dids,
            nfts,
            options,
        };

        for (index, action) in tx.actions.iter().enumerate() {
            action.spend(ctx, &mut spends, index)?;
        }

        let change_amount =
            (selection.xch.existing_amount + summary.created_xch).saturating_sub(summary.spent_xch);

        if change_amount > 0 {
            spends
                .xch
                .create_coin(ctx, change_puzzle_hash, change_amount, false, None)?;
        }

        for (id, cat) in &mut spends.cats {
            let existing_amount = selection
                .cats
                .get(id)
                .map_or(0, |selected| selected.existing_amount);

            let created_amount = summary.created_cats.get(id).copied().unwrap_or_default();
            let spent_amount = summary.spent_cats.get(id).copied().unwrap_or_default();

            let change_amount = (existing_amount + created_amount).saturating_sub(spent_amount);

            if change_amount > 0 {
                cat.create_coin(ctx, change_puzzle_hash, change_amount, true, None)?;
            }
        }

        let xch_spends = spends
            .xch
            .items
            .iter()
            .map(|spend| (spend.coin.coin(), spend.conditions.clone()))
            .collect_vec();

        self.spend_p2_coins_separately(ctx, xch_spends.into_iter())
            .await?;

        for cat in spends.cats.values() {
            let cat_spends = cat
                .items
                .iter()
                .filter_map(|spend| {
                    if let AssetCoin::Cat(cat) = spend.coin {
                        Some((cat, spend.conditions.clone()))
                    } else {
                        None
                    }
                })
                .collect_vec();

            self.spend_cat_coins(ctx, cat_spends.into_iter()).await?;
        }

        for did_item in spends.dids.values_mut() {
            let xch_spends = did_item
                .items
                .iter()
                .filter(|spend| matches!(spend.coin, AssetCoin::Xch(..)))
                .map(|spend| (spend.coin.coin(), spend.conditions.clone()))
                .collect_vec();

            self.spend_p2_coins_separately(ctx, xch_spends.into_iter())
                .await?;

            let (did_spend, did) = did_item.did()?;

            let did = did.update(ctx, &did_spend.p2, did_spend.conditions.clone())?;

            *did_spend = AssetSpend::new(AssetCoin::Did(did), did_spend.p2);
        }

        for nft_item in spends.nfts.values_mut() {
            let xch_spends = nft_item
                .items
                .iter()
                .filter(|spend| matches!(spend.coin, AssetCoin::Xch(..)))
                .map(|spend| (spend.coin.coin(), spend.conditions.clone()))
                .collect_vec();

            self.spend_p2_coins_separately(ctx, xch_spends.into_iter())
                .await?;

            let (nft_spend, nft) = nft_item.nft()?;

            let nft = nft.transfer(
                ctx,
                &nft_spend.p2,
                nft.info.p2_puzzle_hash,
                nft_spend.conditions.clone(),
            )?;

            *nft_spend = AssetSpend::new(AssetCoin::Nft(nft), nft_spend.p2);
        }

        for option_item in spends.options.values_mut() {
            let xch_spends = option_item
                .items
                .iter()
                .filter(|spend| matches!(spend.coin, AssetCoin::Xch(..)))
                .map(|spend| (spend.coin.coin(), spend.conditions.clone()))
                .collect_vec();

            self.spend_p2_coins_separately(ctx, xch_spends.into_iter())
                .await?;

            let (option_spend, option) = option_item.option()?;

            let option = option.transfer(
                ctx,
                &option_spend.p2,
                option.info.p2_puzzle_hash,
                option_spend.conditions.clone(),
            )?;

            *option_spend = AssetSpend::new(AssetCoin::Option(option), option_spend.p2);
        }

        Ok(spends)
    }
}
