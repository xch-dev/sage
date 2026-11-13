use chia_wallet_sdk::prelude::*;

use crate::{ChildKind, CoinKind, WalletError};

#[derive(Debug, Clone)]
pub struct Transaction {
    /// The sum of the transaction's `RESERVE_FEE` conditions.
    pub fee: u64,
    /// The fee the transaction actually pays: the value of the coins it spends minus the
    /// value of the coins it creates, which includes any value left over without a
    /// `RESERVE_FEE` condition. It's `None` when the transaction creates more than it spends,
    /// so isn't a complete transaction on its own (for example, one side of an offer).
    pub actual_fee: Option<u64>,
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
        let mut fee = 0u64;
        let mut spent = 0u64;
        let mut created = 0u64;

        for coin_spend in coin_spends {
            let mut outputs = Vec::new();

            // Coins with no parent are placeholders, such as an offer's requested payments,
            // rather than coins that are actually spent.
            let is_placeholder = coin_spend.coin.parent_coin_info == Bytes32::default();

            if !is_placeholder {
                spent = checked_add(spent, coin_spend.coin.amount)?;
            }

            for condition in run_conditions(&coin_spend.puzzle_reveal, &coin_spend.solution)? {
                match condition {
                    Condition::CreateCoin(create_coin) => {
                        let child_coin = Coin::new(
                            coin_spend.coin.coin_id(),
                            create_coin.puzzle_hash,
                            create_coin.amount,
                        );

                        if !is_placeholder {
                            created = checked_add(created, create_coin.amount)?;
                        }

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
                        fee = checked_add(fee, cond.amount)?;
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

        Ok(Self {
            fee,
            actual_fee: spent.checked_sub(created),
            inputs,
        })
    }
}

fn checked_add(total: u64, amount: u64) -> Result<u64, WalletError> {
    total.checked_add(amount).ok_or(WalletError::AmountOverflow)
}

fn run_conditions(puzzle_reveal: &Program, solution: &Program) -> Result<Conditions, WalletError> {
    let mut allocator = Allocator::new();

    let puzzle = puzzle_reveal.to_clvm(&mut allocator)?;
    let solution = solution.to_clvm(&mut allocator)?;
    let output = run_puzzle(&mut allocator, puzzle, solution)?;
    let conditions = Conditions::<NodePtr>::from_clvm(&allocator, output)?;

    Ok(conditions)
}

#[cfg(test)]
mod tests {
    use chia_wallet_sdk::clvm_traits::clvm_quote;

    use super::*;

    /// A spend of a coin whose puzzle outputs the given conditions.
    fn spend(parent_coin_info: Bytes32, amount: u64, conditions: Conditions) -> CoinSpend {
        let mut ctx = SpendContext::new();
        let puzzle = ctx.alloc(&clvm_quote!(conditions)).unwrap();
        let coin = Coin::new(parent_coin_info, ctx.tree_hash(puzzle).into(), amount);
        CoinSpend::new(
            coin,
            ctx.serialize(&puzzle).unwrap(),
            ctx.serialize(&NodePtr::NIL).unwrap(),
        )
    }

    fn parent() -> Bytes32 {
        Bytes32::new([1; 32])
    }

    fn fees(coin_spends: Vec<CoinSpend>) -> (u64, Option<u64>) {
        let transaction = Transaction::from_coin_spends(coin_spends).unwrap();
        (transaction.fee, transaction.actual_fee)
    }

    #[test]
    fn test_declared_fee() {
        let conditions = Conditions::new()
            .create_coin(Bytes32::default(), 900, Memos::None)
            .reserve_fee(100);

        assert_eq!(
            fees(vec![spend(parent(), 1000, conditions)]),
            (100, Some(100))
        );
    }

    #[test]
    fn test_undeclared_fee() {
        let conditions = Conditions::new().create_coin(Bytes32::default(), 400, Memos::None);
        assert_eq!(
            fees(vec![spend(parent(), 1000, conditions)]),
            (0, Some(600))
        );

        let conditions = Conditions::new()
            .create_coin(Bytes32::default(), 400, Memos::None)
            .reserve_fee(100);
        assert_eq!(
            fees(vec![spend(parent(), 1000, conditions)]),
            (100, Some(600))
        );

        // A coin that creates nothing leaves its whole value as a fee.
        assert_eq!(
            fees(vec![spend(parent(), 1000, Conditions::new())]),
            (0, Some(1000))
        );
    }

    #[test]
    fn test_fee_across_spends() {
        // The second coin creates more than its own value, using what the first leaves over.
        let first = spend(parent(), 1000, Conditions::new());
        let second = spend(
            Bytes32::new([2; 32]),
            100,
            Conditions::new().create_coin(Bytes32::default(), 1050, Memos::None),
        );

        assert_eq!(fees(vec![first, second]), (0, Some(50)));
    }

    #[test]
    fn test_ephemeral_coins_cancel_out() {
        // The second coin is created by the first and spent in the same transaction.
        let first = spend(
            parent(),
            1000,
            Conditions::new().create_coin(Bytes32::default(), 1000, Memos::None),
        );
        let second = spend(
            first.coin.coin_id(),
            1000,
            Conditions::new()
                .create_coin(Bytes32::default(), 990, Memos::None)
                .reserve_fee(10),
        );

        assert_eq!(fees(vec![first, second]), (10, Some(10)));
    }

    #[test]
    fn test_incomplete_transaction() {
        // Creates more than it spends, so the rest must come from spends that aren't here.
        let conditions = Conditions::new()
            .create_coin(Bytes32::default(), 2000, Memos::None)
            .reserve_fee(100);

        assert_eq!(fees(vec![spend(parent(), 1000, conditions)]), (100, None));
    }

    #[test]
    fn test_placeholder_spends_are_ignored() {
        // Like an offer's requested payments: a coin with no parent that "creates" a payment.
        let placeholder = spend(
            Bytes32::default(),
            0,
            Conditions::new().create_coin(Bytes32::default(), 5000, Memos::None),
        );
        let offered = spend(
            parent(),
            1000,
            Conditions::new().create_coin(Bytes32::default(), 1000, Memos::None),
        );

        assert_eq!(fees(vec![placeholder, offered]), (0, Some(0)));
    }

    #[test]
    fn test_overflow() {
        let conditions = Conditions::new().reserve_fee(u64::MAX).reserve_fee(1);
        assert!(matches!(
            Transaction::from_coin_spends(vec![spend(parent(), 1000, conditions)]),
            Err(WalletError::AmountOverflow)
        ));

        let conditions = Conditions::new()
            .create_coin(Bytes32::default(), u64::MAX, Memos::None)
            .create_coin(Bytes32::new([3; 32]), 1, Memos::None);
        assert!(matches!(
            Transaction::from_coin_spends(vec![spend(parent(), 1000, conditions)]),
            Err(WalletError::AmountOverflow)
        ));

        let first = spend(parent(), u64::MAX, Conditions::new());
        let second = spend(Bytes32::new([2; 32]), 1, Conditions::new());
        assert!(matches!(
            Transaction::from_coin_spends(vec![first, second]),
            Err(WalletError::AmountOverflow)
        ));
    }
}
