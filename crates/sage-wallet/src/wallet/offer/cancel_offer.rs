use chia::protocol::CoinSpend;
use chia_wallet_sdk::{
    driver::{HashedPtr, Offer, SpendContext, StandardLayer},
    types::Conditions,
};
use sage_database::CoinKind;

use crate::{Wallet, WalletError};

impl Wallet {
    pub async fn cancel_offer(
        &self,
        offer: Offer,
        fee: u64,
        hardened: bool,
        reuse: bool,
    ) -> Result<Vec<CoinSpend>, WalletError> {
        let mut ctx = SpendContext::new();

        let offer = offer.parse(&mut ctx)?;

        let mut coin_states = Vec::with_capacity(offer.coin_spends.len());

        for coin_spend in offer.coin_spends {
            let Some(row) = self.db.full_coin_state(coin_spend.coin.coin_id()).await? else {
                continue;
            };

            if row.coin_state.created_height.is_none() {
                continue;
            }

            if row.coin_state.spent_height.is_some() {
                return Err(WalletError::UncancellableOffer);
            }

            match row.kind {
                CoinKind::Xch | CoinKind::Cat | CoinKind::Did | CoinKind::Nft => {}
                CoinKind::Unknown => continue,
            }

            coin_states.push(row);
        }

        coin_states.sort_by_key(|row| row.kind);

        let Some(row) = coin_states.first().copied() else {
            return Err(WalletError::UncancellableOffer);
        };

        let coin_state = row.coin_state;

        let mut remaining_fee = fee;
        let mut xch_coins = Vec::new();

        if matches!(row.kind, CoinKind::Xch) {
            xch_coins.push(coin_state.coin);
            remaining_fee = remaining_fee.saturating_sub(coin_state.coin.amount);
        }

        if remaining_fee > 0 {
            xch_coins.extend(self.select_p2_coins(remaining_fee as u128).await?);
        }

        let total_amount = xch_coins.iter().map(|coin| coin.amount).sum::<u64>();
        let change = total_amount - fee;

        let p2_puzzle_hash = self.p2_puzzle_hash(hardened, reuse).await?;

        let mut conditions = Conditions::new();

        if !matches!(row.kind, CoinKind::Xch) {
            conditions = conditions.assert_concurrent_spend(coin_state.coin.coin_id());
        }

        if fee > 0 {
            conditions = conditions.reserve_fee(fee);
        }

        if change > 0 {
            conditions = conditions.create_coin(p2_puzzle_hash, change, None);
        }

        if !xch_coins.is_empty() {
            self.spend_p2_coins(&mut ctx, xch_coins, conditions).await?;
        }

        match row.kind {
            CoinKind::Xch => {}
            CoinKind::Cat => {
                let Some(cat) = self.db.cat_coin(coin_state.coin.coin_id()).await? else {
                    return Err(WalletError::UncancellableOffer);
                };

                let memos = ctx.hint(p2_puzzle_hash)?;

                self.spend_cat_coins(
                    &mut ctx,
                    [(
                        cat,
                        Conditions::new().create_coin(p2_puzzle_hash, cat.coin.amount, Some(memos)),
                    )]
                    .into_iter(),
                )
                .await?;
            }
            CoinKind::Did => {
                let Some(did) = self.db.did_by_coin_id(coin_state.coin.coin_id()).await? else {
                    return Err(WalletError::UncancellableOffer);
                };
                let metadata_ptr = ctx.alloc(&did.info.metadata)?;
                let did = did.with_metadata(HashedPtr::from_ptr(&ctx, metadata_ptr));

                let synthetic_key = self.db.synthetic_key(did.info.p2_puzzle_hash).await?;
                let p2 = StandardLayer::new(synthetic_key);

                let _did = did.transfer(&mut ctx, &p2, p2_puzzle_hash, Conditions::new())?;
            }
            CoinKind::Nft => {
                let Some(nft) = self.db.nft_by_coin_id(coin_state.coin.coin_id()).await? else {
                    return Err(WalletError::UncancellableOffer);
                };
                let metadata_ptr = ctx.alloc(&nft.info.metadata)?;
                let nft = nft.with_metadata(HashedPtr::from_ptr(&ctx, metadata_ptr));

                let synthetic_key = self.db.synthetic_key(nft.info.p2_puzzle_hash).await?;
                let p2 = StandardLayer::new(synthetic_key);

                let _nft = nft.transfer(&mut ctx, &p2, p2_puzzle_hash, Conditions::new())?;
            }
            CoinKind::Unknown => unreachable!(),
        }

        Ok(ctx.take())
    }
}
