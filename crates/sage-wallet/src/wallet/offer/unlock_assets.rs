use std::mem;

use chia::{
    protocol::Bytes32,
    puzzles::offer::{NotarizedPayment, Payment, SettlementPaymentsSolution},
};
use chia_wallet_sdk::{
    payment_assertion, AssertPuzzleAnnouncement, Cat, CatSpend, Layer, SettlementLayer,
    SpendContext,
};

use crate::WalletError;

use super::{LockedCoins, RequestedPayments};

pub fn unlock_assets(
    ctx: &mut SpendContext,
    locked: LockedCoins,
    nonce: Bytes32,
    p2_puzzle_hash: Bytes32,
) -> Result<Vec<AssertPuzzleAnnouncement>, WalletError> {
    let mut assertions = Vec::new();

    for coin in locked.xch {
        let notarized_payment = NotarizedPayment {
            nonce,
            payments: vec![Payment::with_memos(
                p2_puzzle_hash,
                coin.amount,
                vec![p2_puzzle_hash.into()],
            )],
        };

        assertions.push(payment_assertion(coin.puzzle_hash, &notarized_payment));

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

    Ok(assertions)
}

pub fn complete_requested_payments(
    ctx: &mut SpendContext,
    locked: LockedCoins,
    mut requested: RequestedPayments,
) -> Result<(), WalletError> {
    for coin in locked.xch {
        let coin_spend = SettlementLayer.construct_coin_spend(
            ctx,
            coin,
            SettlementPaymentsSolution {
                notarized_payments: mem::take(&mut requested.xch),
            },
        )?;
        ctx.insert(coin_spend);
    }

    for coins in locked.cats.into_values() {
        for cat in coins {
            let inner_spend = SettlementLayer.construct_spend(
                ctx,
                SettlementPaymentsSolution {
                    notarized_payments: mem::take(&mut requested.cats[&cat.asset_id]),
                },
            )?;
            Cat::spend_all(ctx, &[CatSpend::new(cat, inner_spend)])?;
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
