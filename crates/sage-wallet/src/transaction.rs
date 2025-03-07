use chia::{
    clvm_traits::{FromClvm, ToClvm},
    protocol::{Coin, CoinSpend, Program},
};
use chia_wallet_sdk::types::{run_puzzle, Condition, Conditions};
use clvmr::{Allocator, NodePtr};

use crate::{ChildKind, CoinKind, WalletError};

#[derive(Debug, Clone)]
pub struct Transaction {
    pub fee: u64,
    pub inputs: Vec<TransactionInput>,
}

#[derive(Debug, Clone)]
pub struct TransactionInput {
    pub coin_spend: CoinSpend,
    pub kind: CoinKind,
    pub outputs: Vec<TransactionOutput>,
}

#[derive(Debug, Clone)]
pub struct TransactionOutput {
    pub coin: Coin,
    pub kind: ChildKind,
}

impl Transaction {
    pub fn from_coin_spends(coin_spends: Vec<CoinSpend>) -> Result<Self, WalletError> {
        // TODO: Handle height and timestamp conditions.

        let mut inputs = Vec::new();
        let mut fee = 0;

        for coin_spend in coin_spends {
            let mut outputs = Vec::new();

            for condition in run_conditions(&coin_spend.puzzle_reveal, &coin_spend.solution)? {
                match condition {
                    Condition::CreateCoin(create_coin) => {
                        let child_coin = Coin::new(
                            coin_spend.coin.coin_id(),
                            create_coin.puzzle_hash,
                            create_coin.amount,
                        );

                        outputs.push(TransactionOutput {
                            coin: child_coin,
                            kind: ChildKind::from_parent(
                                coin_spend.coin,
                                &coin_spend.puzzle_reveal,
                                &coin_spend.solution,
                                child_coin,
                            )?,
                        });
                    }
                    Condition::ReserveFee(cond) => {
                        fee += cond.amount;
                    }
                    _ => {}
                }
            }

            let kind = CoinKind::from_puzzle(&coin_spend.puzzle_reveal)?;

            inputs.push(TransactionInput {
                coin_spend,
                kind,
                outputs,
            });
        }

        Ok(Self { fee, inputs })
    }
}

fn run_conditions(puzzle_reveal: &Program, solution: &Program) -> Result<Conditions, WalletError> {
    let mut allocator = Allocator::new();

    let puzzle = puzzle_reveal.to_clvm(&mut allocator)?;
    let solution = solution.to_clvm(&mut allocator)?;
    let output = run_puzzle(&mut allocator, puzzle, solution)?;
    let conditions = Conditions::<NodePtr>::from_clvm(&allocator, output)?;

    Ok(conditions)
}
