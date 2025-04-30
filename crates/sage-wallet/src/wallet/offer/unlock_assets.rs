use chia::{
    protocol::{Bytes32, Coin},
    puzzles::offer::{NotarizedPayment, Payment, SettlementPaymentsSolution},
};
use chia_wallet_sdk::{
    driver::{
        payment_assertion, Cat, CatSpend, HashedPtr, Layer, Nft, SettlementLayer, SpendContext,
    },
    types::conditions::AssertPuzzleAnnouncement,
};
use indexmap::IndexMap;

use crate::WalletError;

use super::{make_offer_payments, LockedCoins, PaymentOrigin, RequestedPayments};

#[derive(Debug, Clone)]
pub struct Unlock {
    pub assertions: Vec<AssertPuzzleAnnouncement>,
    pub single_sided_coin: Option<SingleSidedIntermediary>,
}

#[derive(Debug, Clone, Copy)]
pub enum SingleSidedIntermediary {
    Xch(Coin),
    Cat(Cat),
    Nft(Nft<HashedPtr>),
}

pub fn unlock_assets(
    ctx: &mut SpendContext,
    locked: LockedCoins,
    nonce: Bytes32,
    p2_puzzle_hash: Bytes32,
    single_sided: bool,
) -> Result<Unlock, WalletError> {
    let mut assertions = Vec::new();
    let mut intermediary = None;

    let total_xch_amount = locked.xch.iter().map(|coin| coin.amount).sum::<u64>();

    for (i, coin) in locked.xch.into_iter().enumerate() {
        if coin.amount == 0 {
            continue;
        }

        if single_sided && intermediary.is_none() {
            intermediary = Some(SingleSidedIntermediary::Xch(Coin::new(
                coin.coin_id(),
                p2_puzzle_hash,
                total_xch_amount,
            )));
        }

        let notarized_payment = NotarizedPayment {
            nonce,
            payments: if i == 0 {
                vec![Payment::with_memos(
                    p2_puzzle_hash,
                    total_xch_amount,
                    vec![p2_puzzle_hash.into()],
                )]
            } else {
                Vec::new()
            },
        };

        if i == 0 {
            assertions.push(payment_assertion(coin.puzzle_hash, &notarized_payment));
        }

        let coin_spend = SettlementLayer.construct_coin_spend(
            ctx,
            coin,
            SettlementPaymentsSolution {
                notarized_payments: vec![notarized_payment],
            },
        )?;

        ctx.insert(coin_spend);
    }

    for coins in locked.cats.into_values() {
        let mut cat_spends = Vec::new();
        let total_amount = coins.iter().map(|cat| cat.coin.amount).sum::<u64>();

        for (i, cat) in coins.into_iter().enumerate() {
            if cat.coin.amount == 0 {
                continue;
            }

            if single_sided && intermediary.is_none() {
                intermediary = Some(SingleSidedIntermediary::Cat(
                    cat.wrapped_child(p2_puzzle_hash, total_amount),
                ));
            }

            let notarized_payment = NotarizedPayment {
                nonce,
                payments: if i == 0 {
                    vec![Payment::with_memos(
                        p2_puzzle_hash,
                        total_amount,
                        vec![p2_puzzle_hash.into()],
                    )]
                } else {
                    Vec::new()
                },
            };

            if i == 0 {
                assertions.push(payment_assertion(cat.coin.puzzle_hash, &notarized_payment));
            }

            let inner_spend = SettlementLayer.construct_spend(
                ctx,
                SettlementPaymentsSolution {
                    notarized_payments: vec![notarized_payment],
                },
            )?;

            cat_spends.push(CatSpend::new(cat, inner_spend));
        }

        Cat::spend_all(ctx, &cat_spends)?;
    }

    for nft in locked.nfts.into_values() {
        if single_sided && intermediary.is_none() {
            intermediary = Some(SingleSidedIntermediary::Nft(nft.wrapped_child(
                p2_puzzle_hash,
                None,
                nft.info.metadata,
            )));
        }

        let notarized_payment = NotarizedPayment {
            nonce,
            payments: vec![Payment::with_memos(
                p2_puzzle_hash,
                nft.coin.amount,
                vec![p2_puzzle_hash.into()],
            )],
        };

        assertions.push(payment_assertion(nft.coin.puzzle_hash, &notarized_payment));

        let _nft = nft.unlock_settlement(ctx, vec![notarized_payment])?;
    }

    Ok(Unlock {
        assertions,
        single_sided_coin: intermediary,
    })
}

pub fn complete_requested_payments(
    ctx: &mut SpendContext,
    locked: LockedCoins,
    mut requested: RequestedPayments,
) -> Result<(), WalletError> {
    if !locked.xch.is_empty() {
        let mut payments = IndexMap::<Bytes32, Vec<Payment>>::new();

        for notarized_payment in requested.xch {
            for payment in notarized_payment.payments {
                payments
                    .entry(notarized_payment.nonce)
                    .or_default()
                    .push(payment);
            }
        }

        make_offer_payments(
            ctx,
            payments,
            PaymentOrigin::Xch(locked.xch[0]),
            locked.xch[1..]
                .iter()
                .copied()
                .map(PaymentOrigin::Xch)
                .collect(),
        )?;
    }

    for (asset_id, coins) in locked.cats {
        if !coins.is_empty() {
            let mut payments = IndexMap::<Bytes32, Vec<Payment>>::new();

            for notarized_payment in requested.cats[&asset_id].clone() {
                for payment in notarized_payment.payments {
                    payments
                        .entry(notarized_payment.nonce)
                        .or_default()
                        .push(payment);
                }
            }

            make_offer_payments(
                ctx,
                payments,
                PaymentOrigin::Cat(coins[0]),
                coins[1..].iter().copied().map(PaymentOrigin::Cat).collect(),
            )?;
        }
    }

    for nft in locked.nfts.into_values() {
        let _nft = nft.unlock_settlement(
            ctx,
            requested
                .nfts
                .swap_remove(&nft.info.launcher_id)
                .expect("missing NFT")
                .1,
        )?;
    }

    Ok(())
}
