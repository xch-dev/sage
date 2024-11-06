use chia::{
    clvm_utils::CurriedProgram,
    protocol::Bytes32,
    puzzles::{
        cat::CatArgs,
        offer::{Payment, SETTLEMENT_PAYMENTS_PUZZLE_HASH},
    },
};
use chia_wallet_sdk::{
    Conditions, HashedPtr, Layer, NftInfo, Offer, OfferBuilder, SpendContext, StandardLayer,
};
use indexmap::IndexMap;

use crate::{OfferRequest, OfferedCoins, WalletError};

use super::{UnsignedOffer, Wallet};

impl Wallet {
    pub async fn create_offer(
        &self,
        offered: OfferedCoins,
        requested: OfferRequest,
        hardened: bool,
        reuse: bool,
    ) -> Result<UnsignedOffer, WalletError> {
        let p2_puzzle_hash = self.p2_puzzle_hash(hardened, reuse).await?;

        // We need to get a list of all of the coin ids being offered for the nonce.
        let mut coin_ids = Vec::new();

        // Select coins for the XCH being offered.
        let p2_coins = if offered.xch > 0 {
            self.select_p2_coins(offered.xch as u128).await?
        } else {
            Vec::new()
        };

        for p2_coin in &p2_coins {
            coin_ids.push(p2_coin.coin_id());
        }

        // Select coins for the CATs being offered.
        let mut cats = IndexMap::new();

        for (&asset_id, &amount) in &offered.cats {
            if amount == 0 {
                continue;
            }

            let cat_coins = self.select_cat_coins(asset_id, amount as u128).await?;

            for cat_coin in &cat_coins {
                coin_ids.push(cat_coin.coin.coin_id());
            }

            cats.insert(asset_id, cat_coins);
        }

        // Fetch coin info for the NFTs being offered.
        let mut nfts = Vec::new();

        for nft_id in offered.nfts {
            let Some(nft) = self.db.nft(nft_id).await? else {
                return Err(WalletError::MissingNft(nft_id));
            };

            coin_ids.push(nft.coin.coin_id());
            nfts.push(nft);
        }

        // Calculate the nonce for the offer.
        let nonce = Offer::nonce(coin_ids);

        // Create the offer builder with the nonce.
        let mut builder = OfferBuilder::new(nonce);
        let mut ctx = SpendContext::new();
        let settlement = ctx.settlement_payments_puzzle()?;
        let cat = ctx.cat_puzzle()?;

        // Handle requested XCH payments.
        if requested.xch > 0 {
            builder = builder.request(
                &mut ctx,
                &settlement,
                vec![Payment::new(p2_puzzle_hash, requested.xch)],
            )?;
        }

        // Handle requested CAT payments.
        for (asset_id, amount) in requested.cats {
            builder = builder.request(
                &mut ctx,
                &CurriedProgram {
                    program: cat,
                    args: CatArgs::new(asset_id, settlement),
                },
                vec![Payment::with_memos(
                    p2_puzzle_hash,
                    amount,
                    vec![p2_puzzle_hash.into()],
                )],
            )?;
        }

        // Handle requested NFT payments.
        for (nft_id, info) in requested.nfts {
            let info = NftInfo {
                launcher_id: nft_id,
                metadata: info.metadata,
                metadata_updater_puzzle_hash: info.metadata_updater_puzzle_hash,
                current_owner: None,
                royalty_puzzle_hash: info.royalty_puzzle_hash,
                royalty_ten_thousandths: info.royalty_ten_thousandths,
                p2_puzzle_hash: SETTLEMENT_PAYMENTS_PUZZLE_HASH.into(),
            };

            let layers = info.into_layers(settlement).construct_puzzle(&mut ctx)?;

            builder = builder.request(
                &mut ctx,
                &layers,
                vec![Payment::with_memos(
                    p2_puzzle_hash,
                    1,
                    vec![p2_puzzle_hash.into()],
                )],
            )?;
        }

        // Finish the requested payments and get the list of announcement assertions.
        let (assertions, builder) = builder.finish();

        // Spend the XCH being offered.
        if !p2_coins.is_empty() {
            let mut conditions = Conditions::new();

            if offered.xch > 0 {
                conditions = conditions.create_coin(
                    SETTLEMENT_PAYMENTS_PUZZLE_HASH.into(),
                    offered.xch,
                    Vec::new(),
                );
            }

            let total: u128 = p2_coins.iter().map(|coin| coin.amount as u128).sum();
            let change = total - offered.xch as u128;

            if change > 0 {
                conditions = conditions.create_coin(
                    p2_puzzle_hash,
                    change.try_into().expect("change overflow"),
                    Vec::new(),
                );
            }

            for &assertion in &assertions {
                conditions = conditions.with(assertion);
            }

            self.spend_p2_coins(&mut ctx, p2_coins, conditions).await?;
        }

        // Spend the CATs being offered.
        for (asset_id, cat_coins) in cats {
            let total: u128 = cat_coins.iter().map(|cat| cat.coin.amount as u128).sum();
            let amount = offered.cats[&asset_id];
            let change = (total - amount as u128)
                .try_into()
                .expect("change overflow");

            self.spend_cat_coins(
                &mut ctx,
                cat_coins.into_iter().enumerate().map(|(i, cat)| {
                    if i > 0 {
                        return (cat, Conditions::new());
                    }

                    let mut conditions = Conditions::new().create_coin(
                        SETTLEMENT_PAYMENTS_PUZZLE_HASH.into(),
                        amount,
                        vec![Bytes32::from(SETTLEMENT_PAYMENTS_PUZZLE_HASH).into()],
                    );

                    if change > 0 {
                        conditions = conditions.create_coin(
                            p2_puzzle_hash,
                            change,
                            vec![Bytes32::from(p2_puzzle_hash).into()],
                        );
                    }

                    for &assertion in &assertions {
                        conditions = conditions.with(assertion);
                    }

                    (cat, conditions)
                }),
            )
            .await?;
        }

        // Spend the NFTs being offered.
        for nft in nfts {
            let metadata_ptr = ctx.alloc(&nft.info.metadata)?;
            let nft = nft.with_metadata(HashedPtr::from_ptr(&ctx.allocator, metadata_ptr));

            let synthetic_key = self.db.synthetic_key(nft.info.p2_puzzle_hash).await?;
            let p2 = StandardLayer::new(synthetic_key);

            let mut conditions = Conditions::new();

            for &assertion in &assertions {
                conditions = conditions.with(assertion);
            }

            // Add trade prices
            let _ = nft.transfer_to_did(
                &mut ctx,
                &p2,
                SETTLEMENT_PAYMENTS_PUZZLE_HASH.into(),
                None,
                conditions,
            )?;
        }

        // Construct the final offer.
        let coin_spends = ctx.take();

        Ok(UnsignedOffer {
            ctx,
            coin_spends,
            builder,
        })
    }
}
