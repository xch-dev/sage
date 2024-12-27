use chia::protocol::CoinSpend;
use chia_wallet_sdk::{Conditions, Offer, OfferBuilder, SpendContext, Take};
use indexmap::IndexMap;

use crate::{
    calculate_royalties, calculate_trade_prices, complete_requested_payments, parse_locked_coins,
    parse_offer_payments, unlock_assets, NftRoyaltyInfo, OfferAmounts, OfferSpend, Wallet,
    WalletError,
};

#[derive(Debug)]
pub struct UnsignedTakeOffer {
    pub coin_spends: Vec<CoinSpend>,
    pub builder: OfferBuilder<Take>,
}

impl Wallet {
    pub async fn take_offer(
        &self,
        offer: Offer,
        fee: u64,
        hardened: bool,
        reuse: bool,
    ) -> Result<UnsignedTakeOffer, WalletError> {
        let mut ctx = SpendContext::new();

        let offer = offer.parse(&mut ctx.allocator)?;
        let (locked_coins, _original_coins) = parse_locked_coins(&mut ctx.allocator, &offer)?;
        let maker_amounts = locked_coins.amounts();

        let mut builder = offer.take();
        let requested_payments = parse_offer_payments(&mut ctx, &mut builder)?;
        let taker_amounts = requested_payments.amounts();

        let taker_royalties = calculate_royalties(
            &taker_amounts,
            &locked_coins
                .nfts
                .values()
                .map(|nft| NftRoyaltyInfo {
                    launcher_id: nft.info.launcher_id,
                    royalty_puzzle_hash: nft.info.royalty_puzzle_hash,
                    royalty_ten_thousandths: nft.info.royalty_ten_thousandths,
                })
                .collect::<Vec<_>>(),
        )?;

        let total_amounts = taker_amounts.clone()
            + taker_royalties.amounts()
            + OfferAmounts {
                xch: fee,
                cats: IndexMap::new(),
            };
        let taker_coins = self
            .fetch_offer_coins(
                &total_amounts,
                requested_payments.nfts.keys().copied().collect(),
            )
            .await?;
        let p2_puzzle_hash = self.p2_puzzle_hash(hardened, reuse).await?;
        let assertions =
            unlock_assets(&mut ctx, locked_coins, taker_coins.nonce(), p2_puzzle_hash)?;

        // Calculate trade prices for the maker side.
        let trade_prices = calculate_trade_prices(
            &maker_amounts,
            requested_payments
                .nfts
                .values()
                .filter(|(nft, _)| nft.royalty_ten_thousandths > 0)
                .count(),
        )?;

        let extra_conditions = Conditions::new()
            .extend(assertions)
            .extend(taker_royalties.assertions());

        // Spend the assets.
        let payment_coins = self
            .lock_assets(
                &mut ctx,
                OfferSpend {
                    amounts: taker_amounts,
                    coins: taker_coins,
                    royalties: taker_royalties,
                    trade_prices,
                    fee,
                    change_puzzle_hash: p2_puzzle_hash,
                    extra_conditions,
                },
            )
            .await?;

        complete_requested_payments(&mut ctx, payment_coins, requested_payments)?;

        let coin_spends = ctx.take();

        Ok(UnsignedTakeOffer {
            coin_spends,
            builder,
        })
    }
}
