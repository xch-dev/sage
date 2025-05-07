use std::mem;

use chia::protocol::Bytes32;
use chia_wallet_sdk::driver::{OptionLauncherInfo, OptionType, SpendContext};

use crate::{Action, Id, P2Selection, SingletonLineage, Spends, Summary, WalletError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MintOptionAction {
    pub creator_puzzle_hash: Bytes32,
    pub owner_puzzle_hash: Bytes32,
    pub seconds: u64,
    pub underlying_type: OptionTypeWithId,
    pub strike_type: OptionTypeWithId,
}

impl MintOptionAction {
    pub fn new(
        creator_puzzle_hash: Bytes32,
        owner_puzzle_hash: Bytes32,
        seconds: u64,
        underlying_type: OptionTypeWithId,
        strike_type: OptionTypeWithId,
    ) -> Self {
        Self {
            creator_puzzle_hash,
            owner_puzzle_hash,
            seconds,
            underlying_type,
            strike_type,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OptionTypeWithId {
    Xch {
        amount: u64,
    },
    Cat {
        id: Id,
        amount: u64,
    },
    Nft {
        id: Id,
        settlement_puzzle_hash: Bytes32,
    },
}

impl OptionTypeWithId {
    pub fn amount(&self) -> u64 {
        match self {
            Self::Xch { amount } | Self::Cat { amount, .. } => *amount,
            Self::Nft { .. } => 1,
        }
    }

    pub fn resolve(self, spends: &Spends) -> Result<OptionType, WalletError> {
        Ok(match self {
            Self::Xch { amount } => OptionType::Xch { amount },
            Self::Cat { id, amount } => {
                let asset_id = match id {
                    Id::Existing(existing) => existing,
                    Id::New(..) => {
                        spends
                            .cats
                            .get(&id)
                            .ok_or(WalletError::MissingAsset)?
                            .items
                            .first()
                            .ok_or(WalletError::MissingAsset)?
                            .coin
                            .asset_id
                    }
                };

                OptionType::Cat { asset_id, amount }
            }
            Self::Nft {
                id,
                settlement_puzzle_hash,
            } => {
                let launcher_id = match id {
                    Id::Existing(existing) => existing,
                    Id::New(..) => {
                        spends
                            .nfts
                            .get(&id)
                            .ok_or(WalletError::MissingAsset)?
                            .coin()
                            .info
                            .launcher_id
                    }
                };

                OptionType::Nft {
                    launcher_id,
                    settlement_puzzle_hash,
                    amount: 1,
                }
            }
        })
    }
}

impl From<OptionType> for OptionTypeWithId {
    fn from(value: OptionType) -> Self {
        match value {
            OptionType::Xch { amount } => Self::Xch { amount },
            OptionType::Cat { asset_id, amount } => Self::Cat {
                id: Id::Existing(asset_id),
                amount,
            },
            OptionType::Nft {
                launcher_id,
                settlement_puzzle_hash,
                amount: _,
            } => Self::Nft {
                id: Id::Existing(launcher_id),
                settlement_puzzle_hash,
            },
            OptionType::RevocableCat { .. } => unimplemented!(),
        }
    }
}

impl Action for MintOptionAction {
    fn summarize(&self, summary: &mut Summary, index: usize) {
        summary.created_options.insert(Id::New(index));
        summary.spent_xch += 1;

        match self.underlying_type {
            OptionTypeWithId::Xch { amount } => {
                summary.spent_xch += amount;
            }
            OptionTypeWithId::Cat { id, amount } => {
                *summary.spent_cats.entry(id).or_default() += amount;
            }
            OptionTypeWithId::Nft { id, .. } => {
                summary.spent_nfts.insert(id);
            }
        }
    }

    fn spend(
        &self,
        ctx: &mut SpendContext,
        spends: &mut Spends,
        index: usize,
    ) -> Result<(), WalletError> {
        let strike_type = self.strike_type.resolve(spends)?;

        let (item_ref, launcher) = spends.xch.create_option_launcher(
            ctx,
            OptionLauncherInfo::new(
                self.creator_puzzle_hash,
                self.owner_puzzle_hash,
                self.seconds,
                self.underlying_type.amount(),
                strike_type,
            ),
        )?;

        let underlying_coin_id = match self.underlying_type {
            OptionTypeWithId::Xch { amount } => spends
                .xch
                .create_coin(
                    ctx,
                    launcher.p2_puzzle_hash(),
                    amount,
                    false,
                    None,
                    P2Selection::Payment,
                )?
                .coin_id(),
            OptionTypeWithId::Cat { id, amount } => {
                let asset = spends.cats.get_mut(&id).ok_or(WalletError::MissingAsset)?;
                asset
                    .create_coin(
                        ctx,
                        launcher.p2_puzzle_hash(),
                        amount,
                        true,
                        None,
                        P2Selection::Payment,
                    )?
                    .coin
                    .coin_id()
            }
            OptionTypeWithId::Nft { id, .. } => {
                let nft_lineage = spends.nfts.get_mut(&id).ok_or(WalletError::MissingAsset)?;
                nft_lineage.set_p2_puzzle_hash(launcher.p2_puzzle_hash());
                nft_lineage.recreate(ctx)?;
                nft_lineage.coin().coin.coin_id()
            }
        };

        let (mint_option, option) = launcher.with_underlying(underlying_coin_id).mint(ctx)?;

        let item = spends.xch.get_mut(item_ref)?;

        let p2 = item
            .p2
            .as_standard_mut()
            .ok_or(WalletError::P2Unsupported)?;

        p2.conditions = mem::take(&mut p2.conditions).extend(mint_option);

        // TODO: Fix p2
        spends.options.insert(
            Id::New(index),
            SingletonLineage::new(option, item.p2.cleared(), true, false),
        );

        Ok(())
    }
}
