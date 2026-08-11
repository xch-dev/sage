use std::collections::HashSet;

use chia_wallet_sdk::{driver::SingletonLayer, prelude::*};
use indexmap::IndexMap;

use crate::Result;

#[derive(Debug, Clone)]
pub struct OfferExpiration {
    pub expiration_height: Option<u32>,
    pub expiration_timestamp: Option<u64>,
    pub coins: IndexMap<Bytes32, StatusCoinType>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StatusCoinType {
    Settle,
    Cancel { fast_forwardable: bool },
}

pub fn offer_expiration(allocator: &mut Allocator, offer: &Offer) -> Result<OfferExpiration> {
    let mut expiration_height = None::<u32>;
    let mut expiration_timestamp = None::<u64>;
    let mut coins = IndexMap::new();

    let mut non_fast_forwardable = HashSet::new();

    for coin_spend in offer.cancellable_coin_spends()? {
        let puzzle = coin_spend.puzzle_reveal.to_clvm(allocator)?;
        let solution = coin_spend.solution.to_clvm(allocator)?;
        let output = run_puzzle(allocator, puzzle, solution)?;
        let conditions = Vec::<Condition<NodePtr>>::from_clvm(allocator, output)?;

        for condition in conditions.clone() {
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

        let puzzle = Puzzle::parse(allocator, puzzle);

        let mut fast_forwardable = SingletonLayer::<Puzzle>::parse_puzzle(allocator, puzzle)
            .is_ok()
            && coin_spend.coin.amount % 2 == 1;

        let mut has_identical_output = false;

        for condition in conditions.into_iter().skip(2) {
            match condition {
                Condition::AssertMyCoinId(..)
                | Condition::AssertMyParentId(..)
                | Condition::AssertHeightRelative(..)
                | Condition::AssertSecondsRelative(..)
                | Condition::AssertBeforeHeightRelative(..)
                | Condition::AssertBeforeSecondsRelative(..)
                | Condition::AssertMyBirthHeight(..)
                | Condition::AssertMyBirthSeconds(..)
                | Condition::AssertEphemeral(..)
                | Condition::AggSigMe(..)
                | Condition::AggSigParent(..)
                | Condition::AggSigParentAmount(..)
                | Condition::AggSigParentPuzzle(..)
                | Condition::CreateCoinAnnouncement(..) => {
                    fast_forwardable = false;
                }
                Condition::SendMessage(cond) => {
                    let sender = parse_message_flags(cond.mode, MessageSide::Sender);

                    if sender.parent {
                        fast_forwardable = false;
                    }
                }
                Condition::ReceiveMessage(cond) => {
                    let receiver = parse_message_flags(cond.mode, MessageSide::Receiver);

                    if receiver.parent {
                        fast_forwardable = false;
                    }
                }
                Condition::CreateCoin(create_coin) => {
                    if create_coin.amount == coin_spend.coin.amount
                        && create_coin.puzzle_hash == coin_spend.coin.puzzle_hash
                    {
                        has_identical_output = true;
                    }

                    let child_coin_id = Coin::new(
                        coin_spend.coin.coin_id(),
                        create_coin.puzzle_hash,
                        create_coin.amount,
                    )
                    .coin_id();

                    non_fast_forwardable.insert(child_coin_id);
                }
                Condition::AssertConcurrentSpend(cond) => {
                    non_fast_forwardable.insert(cond.coin_id);
                }
                _ => {}
            }
        }

        if !has_identical_output {
            fast_forwardable = false;
        }

        coins.insert(
            coin_spend.coin.coin_id(),
            StatusCoinType::Cancel { fast_forwardable },
        );
    }

    for coin in offer.offered_coins().flatten() {
        coins.insert(coin.coin_id(), StatusCoinType::Settle);
    }

    for (coin_id, coin_type) in &mut coins {
        match coin_type {
            StatusCoinType::Cancel { fast_forwardable } => {
                if non_fast_forwardable.contains(coin_id) {
                    *fast_forwardable = false;
                }
            }
            StatusCoinType::Settle => {}
        }
    }

    Ok(OfferExpiration {
        expiration_height,
        expiration_timestamp,
        coins,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum MessageSide {
    Sender,
    Receiver,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct MessageFlags {
    parent: bool,
    puzzle: bool,
    amount: bool,
}

fn parse_message_flags(mode: u8, side: MessageSide) -> MessageFlags {
    // Get the relevant 3 bits based on direction
    let relevant_bits = if side == MessageSide::Sender {
        (mode & 0b11_1000) >> 3
    } else {
        mode & 0b00_0111
    };

    let mut flags = MessageFlags {
        parent: false,
        puzzle: false,
        amount: false,
    };

    if (relevant_bits & 0b100) != 0 {
        flags.parent = true;
    }

    if (relevant_bits & 0b010) != 0 {
        flags.puzzle = true;
    }

    if (relevant_bits & 0b001) != 0 {
        flags.amount = true;
    }

    flags
}
