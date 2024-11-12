use chia::{
    protocol::Bytes32,
    puzzles::{
        cat::CatArgs,
        offer::{NotarizedPayment, Payment, SETTLEMENT_PAYMENTS_PUZZLE_HASH},
    },
};
use chia_wallet_sdk::{
    calculate_nft_royalty, calculate_nft_trace_price, payment_assertion, AssertPuzzleAnnouncement,
    TradePrice,
};
use indexmap::IndexMap;

use crate::WalletError;

#[derive(Debug, Clone, Copy)]
pub struct NftRoyaltyInfo {
    pub launcher_id: Bytes32,
    pub royalty_puzzle_hash: Bytes32,
    pub royalty_ten_thousandths: u16,
}

#[derive(Debug, Clone, Copy)]
pub struct Royalty {
    pub nft_id: Bytes32,
    pub p2_puzzle_hash: Bytes32,
    pub settlement_puzzle_hash: Bytes32,
    pub amount: u64,
}

pub fn calculate_asset_prices(
    nfts: usize,
    xch: u64,
    cats: &IndexMap<Bytes32, u64>,
) -> Result<Vec<TradePrice>, WalletError> {
    let mut trade_prices = Vec::new();

    if nfts == 0 {
        return Ok(trade_prices);
    }

    for (asset_id, asset_amount) in [(None, xch)].into_iter().chain(
        cats.iter()
            .map(|(asset_id, amount)| (Some(*asset_id), *amount)),
    ) {
        let trade_price =
            calculate_nft_trace_price(asset_amount, nfts).ok_or(WalletError::InvalidTradePrice)?;

        let mut settlement_puzzle_hash = SETTLEMENT_PAYMENTS_PUZZLE_HASH;

        if let Some(asset_id) = asset_id {
            settlement_puzzle_hash = CatArgs::curry_tree_hash(asset_id, settlement_puzzle_hash);
        }

        trade_prices.push(TradePrice {
            puzzle_hash: settlement_puzzle_hash.into(),
            amount: trade_price,
        });
    }

    Ok(trade_prices)
}

pub fn calculate_asset_royalties(
    nfts: &[NftRoyaltyInfo],
    trade_prices: &[TradePrice],
) -> Result<Vec<Royalty>, WalletError> {
    let mut royalties = Vec::new();

    if nfts.is_empty() {
        return Ok(royalties);
    }

    for trade_price in trade_prices {
        for nft in nfts {
            let amount = calculate_nft_royalty(trade_price.amount, nft.royalty_ten_thousandths)
                .ok_or(WalletError::InvalidRoyaltyAmount)?;

            royalties.push(Royalty {
                nft_id: nft.launcher_id,
                p2_puzzle_hash: nft.royalty_puzzle_hash,
                settlement_puzzle_hash: trade_price.puzzle_hash,
                amount,
            });
        }
    }

    Ok(royalties)
}

pub fn calculate_royalty_assertions(royalties: &[Royalty]) -> Vec<AssertPuzzleAnnouncement> {
    royalties
        .iter()
        .map(|royalty| {
            let notarized_payment = NotarizedPayment {
                nonce: royalty.nft_id,
                payments: vec![Payment::with_memos(
                    royalty.p2_puzzle_hash,
                    royalty.amount,
                    vec![royalty.p2_puzzle_hash.into()],
                )],
            };
            payment_assertion(royalty.settlement_puzzle_hash, &notarized_payment)
        })
        .collect()
}
