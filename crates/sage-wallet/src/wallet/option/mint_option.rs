use std::{collections::HashMap, mem};

use chia::{
    protocol::{Bytes32, CoinSpend},
    puzzles::nft::{NftMetadata, NFT_METADATA_UPDATER_PUZZLE_HASH},
};
use chia_wallet_sdk::{
    Conditions, DidOwner, HashedPtr, Launcher, Mod, NftMint, OptionContract, SpendContext,
    StandardLayer,
};
use indexmap::IndexMap;

use crate::{
    calculate_royalties, MakerSide, NftRoyaltyInfo, OfferAmounts, TakerSide, Wallet, WalletError,
};

#[derive(Debug, Clone)]
pub struct Option {
    pub did_id: Bytes32,
    pub maker: MakerSide,
    pub taker: TakerSide,
    pub nft_metadata: NftMetadata,
    pub expiration_seconds: u64,
}

impl Wallet {
    pub async fn mint_option(
        &self,
        Option {
            did_id,
            maker,
            taker,
            nft_metadata,
            expiration_seconds,
        }: Option,
        hardened: bool,
        reuse: bool,
    ) -> Result<Vec<CoinSpend>, WalletError> {
        let ctx = &mut SpendContext::new();

        let Some(did) = self.db.spendable_did(did_id).await? else {
            return Err(WalletError::MissingDid(did_id));
        };

        let p2_puzzle_hash = self.p2_puzzle_hash(hardened, reuse).await?;

        let did_metadata_ptr = ctx.alloc(&did.info.metadata)?;
        let did = did.with_metadata(HashedPtr::from_ptr(&ctx.allocator, did_metadata_ptr));

        let synthetic_key = self.db.synthetic_key(did.info.p2_puzzle_hash).await?;
        let did_p2 = StandardLayer::new(synthetic_key);

        let nft_mint = NftMint {
            metadata: nft_metadata,
            metadata_updater_puzzle_hash: NFT_METADATA_UPDATER_PUZZLE_HASH.into(),
            royalty_puzzle_hash: p2_puzzle_hash,
            royalty_ten_thousandths: 0,
            p2_puzzle_hash,
            owner: Some(DidOwner::from_did_info(&did.info)),
        };

        let (mint_nft, nft) = Launcher::new(did.coin.coin_id(), 0)
            .with_singleton_amount(1)
            .mint_nft(ctx, nft_mint)?;
        let _new_did = did.update(ctx, &did_p2, mint_nft)?;

        // TODO: SPLIT THIS APART, BUT HERE BEGINS THE OFFER CODE
        let maker_amounts = OfferAmounts {
            xch: maker.xch,
            cats: maker.cats.clone(),
        };

        let maker_royalties = calculate_royalties(
            &maker_amounts,
            &taker
                .nfts
                .iter()
                .map(|(nft_id, requested_nft)| NftRoyaltyInfo {
                    launcher_id: *nft_id,
                    royalty_puzzle_hash: requested_nft.royalty_puzzle_hash,
                    royalty_ten_thousandths: requested_nft.royalty_ten_thousandths,
                })
                .collect::<Vec<_>>(),
        )?;

        let total_amounts = maker_amounts.clone()
            + maker_royalties.amounts()
            + OfferAmounts {
                xch: maker.fee + 1,
                cats: IndexMap::new(),
            };

        let coins = self
            .fetch_offer_coins(&total_amounts, maker.nfts.clone())
            .await?;

        let option_contract = OptionContract {
            nft_info: nft.info,
            p2_puzzle_hash,
            expiration_seconds,
        };

        let assertions = Conditions::<HashedPtr>::default();

        let mut extra_conditions = Conditions::new();

        let primary_coins = coins.primary_coin_ids();

        // Calculate conditions for each primary coin.
        let mut primary_conditions = HashMap::new();

        if primary_coins.len() == 1 {
            primary_conditions.insert(primary_coins[0], extra_conditions);
        } else {
            for (i, &coin_id) in primary_coins.iter().enumerate() {
                let relation = if i == 0 {
                    *primary_coins.last().expect("empty primary coins")
                } else {
                    primary_coins[i - 1]
                };

                primary_conditions.insert(
                    coin_id,
                    mem::take(&mut extra_conditions).assert_concurrent_spend(relation),
                );
            }
        }

        // TODO: Keep track of the coins that are locked?

        // Spend the XCH.
        if let Some(primary_xch_coin) = coins.xch.first().copied() {
            let offered_amount = maker_amounts.xch + maker_royalties.xch_amount();

            let mut conditions = primary_conditions
                .remove(&primary_xch_coin.coin_id())
                .unwrap_or_default();

            if offered_amount > 0 {
                let p2_option_puzzle = option_contract.p2_option_puzzle(
                    ctx,
                    offered_amount,
                    assertions.clone(),
                    false,
                )?;

                conditions = conditions.create_coin(
                    p2_option_puzzle.curry_tree_hash().into(),
                    offered_amount,
                    None,
                );
            }

            let total_amount = coins.xch.iter().map(|coin| coin.amount).sum::<u64>();
            let change = total_amount - offered_amount - maker.fee;

            if change > 0 {
                conditions = conditions.create_coin(p2_puzzle_hash, change, None);
            }

            if maker.fee > 0 {
                conditions = conditions.reserve_fee(maker.fee);
            }

            self.spend_p2_coins(ctx, coins.xch, conditions).await?;
        }

        // Spend the CATs.
        for (asset_id, cat_coins) in coins.cats {
            let Some(primary_cat) = cat_coins.first().copied() else {
                continue;
            };

            let offered_amount = maker.cats.get(&asset_id).copied().unwrap_or(0)
                + maker_royalties.cat_amount(asset_id);
            let total_amount = cat_coins.iter().map(|cat| cat.coin.amount).sum::<u64>();
            let change = total_amount - offered_amount;

            let p2_option_puzzle =
                option_contract.p2_option_puzzle(ctx, offered_amount, assertions.clone(), true)?;

            let mut conditions = primary_conditions
                .remove(&primary_cat.coin.coin_id())
                .unwrap_or_default()
                .create_coin(
                    p2_option_puzzle.curry_tree_hash().into(),
                    offered_amount,
                    None,
                );

            if change > 0 {
                let change_hint = ctx.hint(p2_puzzle_hash)?;

                conditions = conditions.create_coin(p2_puzzle_hash, change, Some(change_hint));
            }

            self.spend_cat_coins(
                ctx,
                cat_coins
                    .into_iter()
                    .map(|cat| (cat, mem::take(&mut conditions))),
            )
            .await?;
        }

        // Spend the NFTs.
        for nft in coins.nfts.into_values() {
            let metadata_ptr = ctx.alloc(&nft.info.metadata)?;
            let nft = nft.with_metadata(HashedPtr::from_ptr(&ctx.allocator, metadata_ptr));

            let synthetic_key = self.db.synthetic_key(nft.info.p2_puzzle_hash).await?;
            let p2 = StandardLayer::new(synthetic_key);

            let p2_option_puzzle =
                option_contract.p2_option_puzzle(ctx, nft.coin.amount, assertions.clone(), true)?;

            let conditions = primary_conditions
                .remove(&nft.coin.coin_id())
                .unwrap_or_default();

            let _nft = nft.transfer(
                ctx,
                &p2,
                p2_option_puzzle.curry_tree_hash().into(),
                conditions,
            )?;
        }

        Ok(ctx.take())
    }
}
