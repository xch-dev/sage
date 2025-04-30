use chia::{
    protocol::{Bytes32, CoinSpend, Program},
    puzzles::{cat::CatArgs, offer::Payment},
};
use chia_puzzles::SETTLEMENT_PAYMENT_HASH;
use chia_wallet_sdk::{
    driver::{Layer, NftInfo, OfferBuilder, Partial, SpendContext},
    types::{puzzles::SettlementPayment, Conditions},
};
use indexmap::IndexMap;

use crate::{Wallet, WalletError};

use super::{
    calculate_royalties, calculate_trade_prices, lock_assets::OfferSpend, NftRoyaltyInfo,
    OfferAmounts,
};

#[derive(Debug)]
pub struct UnsignedMakeOffer {
    pub ctx: SpendContext,
    pub coin_spends: Vec<CoinSpend>,
    pub builder: OfferBuilder<Partial>,
}

#[derive(Debug, Default, Clone)]
pub struct MakerSide {
    pub xch: u64,
    pub cats: IndexMap<Bytes32, u64>,
    pub nfts: Vec<Bytes32>,
    pub fee: u64,
    pub p2_puzzle_hash: Option<Bytes32>,
}

#[derive(Debug, Default, Clone)]
pub struct TakerSide {
    pub xch: u64,
    pub cats: IndexMap<Bytes32, u64>,
    pub nfts: IndexMap<Bytes32, RequestedNft>,
}

#[derive(Debug, Clone)]
pub struct RequestedNft {
    pub metadata: Program,
    pub metadata_updater_puzzle_hash: Bytes32,
    pub royalty_puzzle_hash: Bytes32,
    pub royalty_ten_thousandths: u16,
}

impl Wallet {
    pub async fn make_offer(
        &self,
        maker: MakerSide,
        taker: TakerSide,
        expires_at: Option<u64>,
        hardened: bool,
        reuse: bool,
    ) -> Result<UnsignedMakeOffer, WalletError> {
        let maker_amounts = OfferAmounts {
            xch: maker.xch,
            cats: maker.cats,
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
                xch: maker.fee,
                cats: IndexMap::new(),
            };
        let maker_coins = self
            .fetch_offer_coins(&total_amounts, maker.nfts.clone())
            .await?;

        let change_puzzle_hash = self.p2_puzzle_hash(hardened, reuse).await?;
        let p2_puzzle_hash = maker.p2_puzzle_hash.unwrap_or(change_puzzle_hash);

        let mut builder = OfferBuilder::new(maker_coins.nonce());
        let mut ctx = SpendContext::new();
        let settlement = ctx.alloc_mod::<SettlementPayment>()?;

        // Add requested XCH payments.
        if taker.xch > 0 {
            builder = builder.request(
                &mut ctx,
                &settlement,
                vec![Payment::with_memos(
                    p2_puzzle_hash,
                    taker.xch,
                    vec![p2_puzzle_hash.into()],
                )],
            )?;
        }

        // Add requested CAT payments.
        for (&asset_id, &amount) in &taker.cats {
            let cat_puzzle = ctx.curry(CatArgs::new(asset_id, settlement))?;

            builder = builder.request(
                &mut ctx,
                &cat_puzzle,
                vec![Payment::with_memos(
                    p2_puzzle_hash,
                    amount,
                    vec![p2_puzzle_hash.into()],
                )],
            )?;
        }

        // Add requested NFT payments.
        for (nft_id, info) in taker.nfts {
            let info = NftInfo {
                launcher_id: nft_id,
                metadata: info.metadata,
                metadata_updater_puzzle_hash: info.metadata_updater_puzzle_hash,
                current_owner: None,
                royalty_puzzle_hash: info.royalty_puzzle_hash,
                royalty_ten_thousandths: info.royalty_ten_thousandths,
                p2_puzzle_hash: SETTLEMENT_PAYMENT_HASH.into(),
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

        // Calculate trade prices for the taker side.
        let taker_amounts = OfferAmounts {
            xch: taker.xch,
            cats: taker.cats,
        };

        let trade_prices = calculate_trade_prices(
            &taker_amounts,
            maker_coins
                .nfts
                .values()
                .filter(|nft| nft.info.royalty_ten_thousandths > 0)
                .count(),
        )?;

        let (assertions, builder) = builder.finish();
        let mut extra_conditions = Conditions::new()
            .extend(assertions)
            .extend(maker_royalties.assertions());

        if let Some(expires_at) = expires_at {
            extra_conditions = extra_conditions.assert_before_seconds_absolute(expires_at);
        }

        // Spend the assets.
        self.lock_assets(
            &mut ctx,
            OfferSpend {
                amounts: maker_amounts,
                coins: maker_coins,
                royalties: maker_royalties,
                trade_prices,
                fee: maker.fee,
                change_puzzle_hash,
                extra_conditions,
            },
        )
        .await?;

        let coin_spends = ctx.take();

        Ok(UnsignedMakeOffer {
            ctx,
            coin_spends,
            builder,
        })
    }
}
