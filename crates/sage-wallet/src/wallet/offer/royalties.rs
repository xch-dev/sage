use chia::{
    protocol::Bytes32,
    puzzles::{
        cat::CatArgs,
        offer::{NotarizedPayment, Payment},
    },
};
use chia_puzzles::SETTLEMENT_PAYMENT_HASH;
use chia_wallet_sdk::{
    driver::{calculate_nft_royalty, calculate_nft_trace_price, payment_assertion},
    prelude::{AssertPuzzleAnnouncement, TradePrice},
};
use indexmap::IndexMap;

use crate::WalletError;

use super::OfferAmounts;

#[derive(Debug, Default, Clone)]
pub struct Royalties {
    pub xch: Vec<RoyaltyPayment>,
    pub cats: IndexMap<Bytes32, Vec<RoyaltyPayment>>,
}

impl Royalties {
    pub fn xch_amount(&self) -> u64 {
        self.xch.iter().map(|royalty| royalty.amount).sum()
    }

    pub fn cat_amount(&self, asset_id: Bytes32) -> u64 {
        self.cats.get(&asset_id).map_or(0, |royalties| {
            royalties.iter().map(|royalty| royalty.amount).sum()
        })
    }

    pub fn amounts(&self) -> OfferAmounts {
        let mut amounts = OfferAmounts {
            xch: self.xch_amount(),
            ..Default::default()
        };

        for &asset_id in self.cats.keys() {
            amounts.cats.insert(asset_id, self.cat_amount(asset_id));
        }

        amounts
    }

    pub fn assertions(&self) -> Vec<AssertPuzzleAnnouncement> {
        let mut assertions = Vec::new();

        for royalty in &self.xch {
            assertions.push(payment_assertion(
                SETTLEMENT_PAYMENT_HASH.into(),
                &royalty.notarized_payment(),
            ));
        }

        for (&asset_id, royalties) in &self.cats {
            for royalty in royalties {
                assertions.push(payment_assertion(
                    CatArgs::curry_tree_hash(asset_id, SETTLEMENT_PAYMENT_HASH.into()).into(),
                    &royalty.notarized_payment(),
                ));
            }
        }

        assertions
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RoyaltyPayment {
    pub nft_id: Bytes32,
    pub p2_puzzle_hash: Bytes32,
    pub amount: u64,
}

impl RoyaltyPayment {
    pub fn notarized_payment(&self) -> NotarizedPayment {
        NotarizedPayment {
            nonce: self.nft_id,
            payments: vec![Payment::with_memos(
                self.p2_puzzle_hash,
                self.amount,
                vec![self.p2_puzzle_hash.into()],
            )],
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct NftRoyaltyInfo {
    pub launcher_id: Bytes32,
    pub royalty_puzzle_hash: Bytes32,
    pub royalty_ten_thousandths: u16,
}

pub fn calculate_trade_prices(
    amounts: &OfferAmounts,
    nft_count: usize,
) -> Result<Vec<TradePrice>, WalletError> {
    let mut trade_prices = Vec::new();

    if nft_count == 0 {
        return Ok(trade_prices);
    }

    if amounts.xch > 0 {
        let amount = calculate_nft_trace_price(amounts.xch, nft_count)
            .ok_or(WalletError::InvalidTradePrice)?;

        trade_prices.push(TradePrice {
            amount,
            puzzle_hash: SETTLEMENT_PAYMENT_HASH.into(),
        });
    }

    for (&asset_id, &amount) in &amounts.cats {
        let amount =
            calculate_nft_trace_price(amount, nft_count).ok_or(WalletError::InvalidTradePrice)?;

        trade_prices.push(TradePrice {
            amount,
            puzzle_hash: CatArgs::curry_tree_hash(asset_id, SETTLEMENT_PAYMENT_HASH.into()).into(),
        });
    }

    Ok(trade_prices)
}

pub fn calculate_royalties(
    amounts: &OfferAmounts,
    nfts: &[NftRoyaltyInfo],
) -> Result<Royalties, WalletError> {
    let royalty_nfts = nfts
        .iter()
        .filter(|nft| nft.royalty_ten_thousandths > 0)
        .count();

    let mut royalties = Royalties::default();

    if royalty_nfts == 0 {
        return Ok(royalties);
    }

    if amounts.xch > 0 {
        let trade_price = calculate_nft_trace_price(amounts.xch, royalty_nfts)
            .ok_or(WalletError::InvalidTradePrice)?;

        for nft in nfts {
            if nft.royalty_ten_thousandths == 0 {
                continue;
            }

            let amount = calculate_nft_royalty(trade_price, nft.royalty_ten_thousandths)
                .ok_or(WalletError::InvalidRoyaltyAmount)?;

            if amount == 0 {
                continue;
            }

            royalties.xch.push(RoyaltyPayment {
                nft_id: nft.launcher_id,
                p2_puzzle_hash: nft.royalty_puzzle_hash,
                amount,
            });
        }
    }

    for (&asset_id, &amount) in &amounts.cats {
        let trade_price = calculate_nft_trace_price(amount, royalty_nfts)
            .ok_or(WalletError::InvalidTradePrice)?;

        for nft in nfts {
            if nft.royalty_ten_thousandths == 0 {
                continue;
            }

            let amount = calculate_nft_royalty(trade_price, nft.royalty_ten_thousandths)
                .ok_or(WalletError::InvalidRoyaltyAmount)?;

            if amount == 0 {
                continue;
            }

            royalties
                .cats
                .entry(asset_id)
                .or_default()
                .push(RoyaltyPayment {
                    nft_id: nft.launcher_id,
                    p2_puzzle_hash: nft.royalty_puzzle_hash,
                    amount,
                });
        }
    }

    Ok(royalties)
}
