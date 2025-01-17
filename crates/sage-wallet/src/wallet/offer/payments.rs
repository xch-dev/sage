use chia::{
    protocol::{Bytes32, Coin},
    puzzles::offer::{
        NotarizedPayment, Payment, SettlementPaymentsSolution, SETTLEMENT_PAYMENTS_PUZZLE_HASH,
    },
};
use chia_wallet_sdk::{
    Cat, CatSpend, CreateCoin, DriverError, Layer, SettlementLayer, SpendContext,
};
use clvmr::NodePtr;

use super::RoyaltyPayment;

#[must_use]
#[derive(Debug, Clone, Copy)]
pub enum RoyaltyOrigin {
    Xch(Coin),
    Cat(Cat),
}

impl RoyaltyOrigin {
    pub fn descendent(&self, p2_puzzle_hash: Bytes32, remaining_amount: u64) -> Self {
        match self {
            Self::Xch(coin) => {
                Self::Xch(Coin::new(coin.coin_id(), p2_puzzle_hash, remaining_amount))
            }
            Self::Cat(cat) => Self::Cat(cat.wrapped_child(p2_puzzle_hash, remaining_amount)),
        }
    }

    pub fn coin(&self) -> Coin {
        match self {
            Self::Xch(coin) => *coin,
            Self::Cat(cat) => cat.coin,
        }
    }
}

pub fn make_royalty_payments(
    ctx: &mut SpendContext,
    total_amount: u64,
    royalties: Vec<RoyaltyPayment>,
    origin: RoyaltyOrigin,
) -> Result<CreateCoin<NodePtr>, DriverError> {
    let mut parent_coin = origin.descendent(SETTLEMENT_PAYMENTS_PUZZLE_HASH.into(), total_amount);
    let mut remaining_payments = royalties.into_iter().rev().collect::<Vec<_>>();
    let mut cat_spends = Vec::new();

    while !remaining_payments.is_empty() {
        let mut outputs = Vec::new();
        let mut notarized_payments = Vec::new();

        for (i, payment) in remaining_payments.clone().into_iter().enumerate().rev() {
            let payment_coin = parent_coin
                .descendent(payment.p2_puzzle_hash, payment.amount)
                .coin();

            if outputs.contains(&payment_coin) {
                continue;
            }

            remaining_payments.remove(i);

            notarized_payments.push(NotarizedPayment {
                nonce: payment.nft_id,
                payments: vec![Payment::with_memos(
                    payment.p2_puzzle_hash,
                    payment.amount,
                    vec![payment.p2_puzzle_hash.into()],
                )],
            });

            outputs.push(payment_coin);
        }

        let remaining_amount = remaining_payments
            .iter()
            .map(|royalty| royalty.amount)
            .sum::<u64>();

        if !remaining_payments.is_empty() {
            notarized_payments.push(NotarizedPayment {
                // TODO: Make nonce nil as an optimization
                nonce: Bytes32::default(),
                payments: vec![Payment::new(
                    SETTLEMENT_PAYMENTS_PUZZLE_HASH.into(),
                    remaining_amount,
                )],
            });
        }

        let spend = SettlementLayer
            .construct_spend(ctx, SettlementPaymentsSolution { notarized_payments })?;

        match parent_coin {
            RoyaltyOrigin::Xch(coin) => ctx.spend(coin, spend)?,
            RoyaltyOrigin::Cat(cat) => cat_spends.push(CatSpend::new(cat, spend)),
        }

        parent_coin =
            parent_coin.descendent(SETTLEMENT_PAYMENTS_PUZZLE_HASH.into(), remaining_amount);
    }

    if !cat_spends.is_empty() {
        Cat::spend_all(ctx, &cat_spends)?;
    }

    Ok(CreateCoin::new(
        SETTLEMENT_PAYMENTS_PUZZLE_HASH.into(),
        total_amount,
        None,
    ))
}
