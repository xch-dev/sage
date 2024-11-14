use chia::{
    clvm_traits::{FromClvm, ToClvm},
    protocol::{Bytes32, Coin},
    puzzles::offer::{NotarizedPayment, SETTLEMENT_PAYMENTS_PUZZLE_HASH},
};
use chia_wallet_sdk::{
    run_puzzle, Cat, CatLayer, Condition, Conditions, HashedPtr, Layer, Nft, NftInfo, OfferBuilder,
    ParsedOffer, Puzzle, SpendContext, Take,
};
use clvmr::{Allocator, NodePtr};
use indexmap::IndexMap;

use crate::WalletError;

use super::OfferAmounts;

#[derive(Debug, Default, Clone)]
pub struct LockedCoins {
    pub xch: Vec<Coin>,
    pub cats: IndexMap<Bytes32, Vec<Cat>>,
    pub nfts: IndexMap<Bytes32, Nft<HashedPtr>>,
    pub fee: u64,
}

impl LockedCoins {
    pub fn amounts(&self) -> OfferAmounts {
        let mut xch = 0;

        for coin in &self.xch {
            xch += coin.amount;
        }

        let mut cats = IndexMap::new();

        for (asset_id, coins) in &self.cats {
            let mut amount = 0;

            for cat in coins {
                amount += cat.coin.amount;
            }

            cats.insert(*asset_id, amount);
        }

        OfferAmounts { xch, cats }
    }
}

#[derive(Debug, Clone)]
pub struct RequestedPayments {
    pub xch: Vec<NotarizedPayment>,
    pub cats: IndexMap<Bytes32, Vec<NotarizedPayment>>,
    pub nfts: IndexMap<Bytes32, (NftInfo<HashedPtr>, Vec<NotarizedPayment>)>,
}

impl RequestedPayments {
    pub fn amounts(&self) -> OfferAmounts {
        let mut xch = 0;

        for item in &self.xch {
            for payment in &item.payments {
                xch += payment.amount;
            }
        }

        let mut cats = IndexMap::new();

        for (asset_id, payments) in &self.cats {
            let mut amount = 0;

            for item in payments {
                for payment in &item.payments {
                    amount += payment.amount;
                }
            }

            cats.insert(*asset_id, amount);
        }

        OfferAmounts { xch, cats }
    }
}

pub fn parse_locked_coins(
    allocator: &mut Allocator,
    offer: &ParsedOffer,
) -> Result<LockedCoins, WalletError> {
    let mut xch = Vec::new();
    let mut cats = IndexMap::new();
    let mut nfts = IndexMap::new();
    let mut fee = 0;

    for coin_spend in &offer.coin_spends {
        let puzzle = coin_spend.puzzle_reveal.to_clvm(allocator)?;
        let puzzle = Puzzle::parse(allocator, puzzle);
        let solution = coin_spend.solution.to_clvm(allocator)?;

        let output = run_puzzle(allocator, puzzle.ptr(), solution)?;
        let conditions = Conditions::<NodePtr>::from_clvm(allocator, output)?;

        let mut coins = Vec::new();

        for condition in conditions {
            match condition {
                Condition::ReserveFee(cond) => fee += cond.amount,
                Condition::CreateCoin(cond) => coins.push(Coin::new(
                    coin_spend.coin.coin_id(),
                    cond.puzzle_hash,
                    cond.amount,
                )),
                _ => {}
            }
        }

        for coin in coins {
            if coin.puzzle_hash == SETTLEMENT_PAYMENTS_PUZZLE_HASH.into() {
                xch.push(coin);
            }
        }

        if let Some(children) = Cat::parse_children(allocator, coin_spend.coin, puzzle, solution)? {
            for child in children {
                if child.p2_puzzle_hash == SETTLEMENT_PAYMENTS_PUZZLE_HASH.into() {
                    cats.entry(child.asset_id)
                        .or_insert_with(Vec::new)
                        .push(child);
                }
            }
        }

        if let Some(child) =
            Nft::<HashedPtr>::parse_child(allocator, coin_spend.coin, puzzle, solution)?
        {
            if child.info.p2_puzzle_hash == SETTLEMENT_PAYMENTS_PUZZLE_HASH.into() {
                nfts.insert(child.info.launcher_id, child);
            }
        }
    }

    Ok(LockedCoins {
        xch,
        cats,
        nfts,
        fee,
    })
}

pub fn parse_offer_payments(
    ctx: &mut SpendContext,
    builder: &mut OfferBuilder<Take>,
) -> Result<RequestedPayments, WalletError> {
    let mut xch_payments = Vec::new();
    let mut cat_payments = IndexMap::new();
    let mut nft_payments = IndexMap::new();

    while let Some((puzzle, payments)) = builder.fulfill() {
        if let Some(cat) = CatLayer::<Puzzle>::parse_puzzle(&ctx.allocator, puzzle)? {
            if cat.inner_puzzle.curried_puzzle_hash() != SETTLEMENT_PAYMENTS_PUZZLE_HASH {
                return Err(WalletError::InvalidRequestedPayment);
            }

            cat_payments
                .entry(cat.asset_id)
                .or_insert_with(Vec::new)
                .extend(payments);
        } else if let Some((nft_info, inner_puzzle)) =
            NftInfo::<HashedPtr>::parse(&ctx.allocator, puzzle)?
        {
            if inner_puzzle.curried_puzzle_hash() != SETTLEMENT_PAYMENTS_PUZZLE_HASH {
                return Err(WalletError::InvalidRequestedPayment);
            }

            if nft_payments
                .insert(nft_info.launcher_id, (nft_info, payments))
                .is_some()
            {
                return Err(WalletError::DuplicateNftRequestedPayment(
                    nft_info.launcher_id,
                ));
            }
        } else if puzzle.curried_puzzle_hash() == SETTLEMENT_PAYMENTS_PUZZLE_HASH {
            xch_payments.extend(payments);
        } else {
            return Err(WalletError::UnknownRequestedPayment(
                puzzle.mod_hash().into(),
            ));
        }
    }

    Ok(RequestedPayments {
        xch: xch_payments,
        cats: cat_payments,
        nfts: nft_payments,
    })
}
