use chia::clvm_traits::{FromClvm, ToClvm};
use chia_wallet_sdk::{
    driver::Offer,
    types::{run_puzzle, Condition},
};
use clvmr::{Allocator, NodePtr};

use crate::Result;

#[derive(Debug, Clone)]
pub struct OfferExpiration {
    pub expiration_height: Option<u32>,
    pub expiration_timestamp: Option<u64>,
}

pub fn offer_expiration(allocator: &mut Allocator, offer: &Offer) -> Result<OfferExpiration> {
    let mut expiration_height = None::<u32>;
    let mut expiration_timestamp = None::<u64>;

    for coin_spend in offer.cancellable_coin_spends()? {
        let puzzle = coin_spend.puzzle_reveal.to_clvm(allocator)?;
        let solution = coin_spend.solution.to_clvm(allocator)?;
        let output = run_puzzle(allocator, puzzle, solution)?;
        let conditions = Vec::<Condition<NodePtr>>::from_clvm(allocator, output)?;

        for condition in conditions {
            match condition {
                Condition::AssertBeforeHeightAbsolute(cond) => {
                    expiration_height = if let Some(old_height) = expiration_height {
                        Some(old_height.min(cond.height))
                    } else {
                        Some(cond.height)
                    };
                }
                Condition::AssertBeforeSecondsAbsolute(cond) => {
                    expiration_timestamp = if let Some(old_timestamp) = expiration_timestamp {
                        Some(old_timestamp.min(cond.seconds))
                    } else {
                        Some(cond.seconds)
                    };
                }
                _ => {}
            }
        }
    }

    Ok(OfferExpiration {
        expiration_height,
        expiration_timestamp,
    })
}
