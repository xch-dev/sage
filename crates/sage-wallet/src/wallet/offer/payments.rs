use chia::{
    protocol::{Bytes32, Coin},
    puzzles::offer::{NotarizedPayment, Payment, SettlementPaymentsSolution},
};
use chia_puzzles::SETTLEMENT_PAYMENT_HASH;
use chia_wallet_sdk::{
    driver::{Cat, CatSpend, DriverError, Layer, SettlementLayer, SpendContext},
    types::conditions::CreateCoin,
};
use clvmr::NodePtr;
use indexmap::IndexMap;

use super::RoyaltyPayment;

#[must_use]
#[derive(Debug, Clone, Copy)]
pub enum PaymentOrigin {
    Xch(Coin),
    Cat(Cat),
}

impl PaymentOrigin {
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
    origin: PaymentOrigin,
) -> Result<CreateCoin<NodePtr>, DriverError> {
    let mut parent_coin = origin.descendent(SETTLEMENT_PAYMENT_HASH.into(), total_amount);
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
                    SETTLEMENT_PAYMENT_HASH.into(),
                    remaining_amount,
                )],
            });
        }

        let spend = SettlementLayer
            .construct_spend(ctx, SettlementPaymentsSolution { notarized_payments })?;

        match parent_coin {
            PaymentOrigin::Xch(coin) => ctx.spend(coin, spend)?,
            PaymentOrigin::Cat(cat) => cat_spends.push(CatSpend::new(cat, spend)),
        }

        parent_coin = parent_coin.descendent(SETTLEMENT_PAYMENT_HASH.into(), remaining_amount);
    }

    if !cat_spends.is_empty() {
        Cat::spend_all(ctx, &cat_spends)?;
    }

    Ok(CreateCoin::new(
        SETTLEMENT_PAYMENT_HASH.into(),
        total_amount,
        None,
    ))
}

pub fn make_offer_payments(
    ctx: &mut SpendContext,
    payments: IndexMap<Bytes32, Vec<Payment>>,
    origin: PaymentOrigin,
    other_coins: Vec<PaymentOrigin>,
) -> Result<(), DriverError> {
    let mut parent_coin = origin;
    let mut remaining_payments = payments.into_iter().rev().collect::<IndexMap<_, _>>();
    let mut cat_spends = Vec::new();

    let mut first = true;

    while !remaining_payments.is_empty() || first {
        first = false;

        let mut outputs = Vec::new();
        let mut notarized_payments = Vec::new();

        let mut overflow_payments = IndexMap::<Bytes32, Vec<Payment>>::new();

        while let Some((nonce, payments)) = remaining_payments.pop() {
            let mut new_payment_coins = Vec::new();
            let mut notarized_payment = NotarizedPayment {
                nonce,
                payments: Vec::new(),
            };

            for payment in payments {
                let payment_coin = parent_coin
                    .descendent(payment.puzzle_hash, payment.amount)
                    .coin();

                if outputs.contains(&payment_coin) {
                    overflow_payments.entry(nonce).or_default().push(payment);
                    continue;
                }

                new_payment_coins.push(payment_coin);
                notarized_payment.payments.push(payment);
            }

            notarized_payments.push(notarized_payment);

            outputs.extend(new_payment_coins);
        }

        remaining_payments.extend(overflow_payments);

        let remaining_amount = remaining_payments
            .iter()
            .map(|notarized_payment| {
                notarized_payment
                    .1
                    .iter()
                    .map(|payment| payment.amount)
                    .sum::<u64>()
            })
            .sum::<u64>();

        if !remaining_payments.is_empty() {
            notarized_payments.push(NotarizedPayment {
                // TODO: Make nonce nil as an optimization
                nonce: Bytes32::default(),
                payments: vec![Payment::new(
                    SETTLEMENT_PAYMENT_HASH.into(),
                    remaining_amount,
                )],
            });
        }

        let spend = SettlementLayer
            .construct_spend(ctx, SettlementPaymentsSolution { notarized_payments })?;

        match parent_coin {
            PaymentOrigin::Xch(coin) => ctx.spend(coin, spend)?,
            PaymentOrigin::Cat(cat) => cat_spends.push(CatSpend::new(cat, spend)),
        }

        parent_coin = parent_coin.descendent(SETTLEMENT_PAYMENT_HASH.into(), remaining_amount);
    }

    for other_coin in other_coins {
        let spend = SettlementLayer.construct_spend(
            ctx,
            SettlementPaymentsSolution {
                notarized_payments: Vec::new(),
            },
        )?;

        match other_coin {
            PaymentOrigin::Xch(coin) => ctx.spend(coin, spend)?,
            PaymentOrigin::Cat(cat) => cat_spends.push(CatSpend::new(cat, spend)),
        }
    }

    if !cat_spends.is_empty() {
        Cat::spend_all(ctx, &cat_spends)?;
    }

    Ok(())
}
