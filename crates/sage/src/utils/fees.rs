use std::borrow::Cow;

use chia_wallet_sdk::{
    chia::consensus::{
        consensus_constants::ConsensusConstants, owned_conditions::OwnedSpendBundleConditions,
    },
    driver::{MIN_FEE_PER_COST, spend_bundle_conditions, spend_bundle_cost},
    prelude::*,
};
use sage_config::InheritedNetwork;
use sage_wallet::{PreSplitOffer, TakenOffer};

use crate::{Error, Result, Sage};

/// The most times a transaction is rebuilt to find a fee that covers its cost. Each rebuild
/// raises the fee by what the previous attempt was missing, and the fee never goes down, so it
/// normally settles within two or three attempts.
const MAX_FEE_ATTEMPTS: usize = 8;

/// The fee rate, in mojos per cost, that a transaction is priced at when the wallet calculates
/// its fee. It's the minimum at which farmers that enforce a minimum block fee include it in a
/// block, and a full mempool admits it.
pub const RECOMMENDED_FEE_PER_COST: u64 = MIN_FEE_PER_COST;

/// A transaction built by [`build_with_fee`], which can be priced from its coin spends.
pub(crate) trait PricedTransaction {
    /// Every coin spend of the transaction as it will be submitted, so that its cost and the
    /// fee it pays can be calculated. It's empty if there's no transaction to pay for.
    fn priced_coin_spends(&self) -> Cow<'_, [CoinSpend]>;
}

impl PricedTransaction for Vec<CoinSpend> {
    fn priced_coin_spends(&self) -> Cow<'_, [CoinSpend]> {
        Cow::Borrowed(self)
    }
}

impl<T> PricedTransaction for (Vec<CoinSpend>, T) {
    fn priced_coin_spends(&self) -> Cow<'_, [CoinSpend]> {
        Cow::Borrowed(&self.0)
    }
}

/// A taken offer is submitted together with the maker's spends, which pay for part of the cost
/// and may pay part of the fee.
impl PricedTransaction for TakenOffer {
    fn priced_coin_spends(&self) -> Cow<'_, [CoinSpend]> {
        Cow::Owned(
            [
                self.offer.spend_bundle().coin_spends.as_slice(),
                self.spend_bundle.coin_spends.as_slice(),
            ]
            .concat(),
        )
    }
}

/// Only the transaction that splits an offer's coins is priced, since the offer's own cost
/// isn't known until it's taken. If nothing needed splitting, there's nothing to pay for.
impl PricedTransaction for PreSplitOffer {
    fn priced_coin_spends(&self) -> Cow<'_, [CoinSpend]> {
        Cow::Borrowed(self.split_coin_spends.as_deref().unwrap_or_default())
    }
}

/// How a transaction's cost is calculated: the consensus rules of the network, and the height
/// the transaction is expected to enter the mempool at.
#[derive(Debug, Clone, Copy)]
pub(crate) struct CostRules {
    constants: &'static ConsensusConstants,
    height: Option<u32>,
}

impl CostRules {
    /// The cost the mempool charges for the coin spends, and the fee they pay.
    fn price(&self, coin_spends: &[CoinSpend]) -> Result<(u64, u64)> {
        let conditions = |height| spend_bundle_conditions(coin_spends, self.constants, height);

        // Without a height, the cost is the higher of the costs on either side of the 3.0 hard
        // fork, as `spend_bundle_cost` prices it. The fee is the same on both sides.
        let fork_height = self.constants.hard_fork2_height;
        let (cost, conditions) = if let Some(height) = self.height {
            let conditions = conditions(height)?;
            (conditions.cost, conditions)
        } else {
            let after = conditions(fork_height)?;
            let before = match fork_height.checked_sub(1) {
                Some(height) => conditions(height)?.cost,
                None => after.cost,
            };
            (before.max(after.cost), after)
        };

        Ok((cost, paid_fee(&conditions)?))
    }

    /// The cost the mempool charges for the coin spends, or `None` if they aren't a complete
    /// transaction, such as one side of an offer.
    pub(crate) fn cost(&self, coin_spends: &[CoinSpend]) -> Option<u64> {
        spend_bundle_cost(coin_spends, self.constants, self.height).ok()
    }
}

/// The fee a transaction pays: the value of the coins it spends minus the value of those it
/// creates.
fn paid_fee(conditions: &OwnedSpendBundleConditions) -> Result<u64> {
    conditions
        .removal_amount
        .checked_sub(conditions.addition_amount)
        .and_then(|fee| u64::try_from(fee).ok())
        .ok_or_else(|| {
            Error::InvalidAmount("the transaction creates more than it spends".to_string())
        })
}

/// Returns the fee, in mojos, for a transaction with the given cost at the given rate in mojos
/// per cost, rounded up.
fn fee_for_cost(cost: u64, fee_per_cost: f64) -> u64 {
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss
    )]
    let fee = (cost as f64 * fee_per_cost).ceil() as u64;
    fee
}

impl Sage {
    /// The rules a transaction created now is priced with. The cost depends on the height
    /// because the 3.0 hard fork changes it, so when no peak is known yet, the transaction is
    /// priced at the higher of the costs on either side of the fork.
    pub(crate) async fn cost_rules(&self) -> Result<CostRules> {
        let network = self.network();

        let (constants, known_network) = match network.same_chain_as() {
            Some(InheritedNetwork::Mainnet) => (&MAINNET_CONSTANTS, true),
            Some(InheritedNetwork::Testnet11) => (&TESTNET11_CONSTANTS, true),
            None => (&MAINNET_CONSTANTS, false),
        };

        // The fork heights of a network that isn't mainnet or testnet11 aren't known, so it's
        // priced at the higher of the two costs whatever its height.
        let height = if known_network {
            let peak = self.peer_state.lock().await.peak();

            match peak {
                Some((height, _)) => Some(height),
                None => self
                    .wallet()?
                    .db
                    .latest_peak()
                    .await?
                    .map(|(height, _)| height),
            }
        } else {
            None
        };

        Ok(CostRules { constants, height })
    }

    /// Starts pricing a transaction with a fee of `fee`, and at `fee_per_cost` mojos per cost
    /// if it's set. Use it through [`build_with_fee`].
    pub(crate) async fn fee_pricer(
        &self,
        fee: u64,
        fee_per_cost: Option<f64>,
    ) -> Result<FeePricer> {
        let Some(fee_per_cost) = fee_per_cost else {
            return Ok(FeePricer {
                fee,
                rate: None,
                attempts: 0,
            });
        };

        if !fee_per_cost.is_finite() || fee_per_cost < 0.0 {
            return Err(Error::InvalidAmount(format!(
                "fee per cost must be a non-negative number, not {fee_per_cost}"
            )));
        }

        Ok(FeePricer {
            fee,
            rate: Some((fee_per_cost, self.cost_rules().await?)),
            attempts: 0,
        })
    }
}

/// Finds the fee for a transaction that's rebuilt until its fee covers its cost. See
/// [`build_with_fee`].
#[derive(Debug, Clone, Copy)]
pub(crate) struct FeePricer {
    fee: u64,
    rate: Option<(f64, CostRules)>,
    attempts: usize,
}

impl FeePricer {
    /// The fee to build the next attempt with.
    pub(crate) fn fee(&self) -> u64 {
        self.fee
    }

    /// Returns whether the transaction's fee covers its cost. If it doesn't, the fee is raised
    /// by the difference for the next attempt.
    pub(crate) fn covers(&mut self, transaction: &impl PricedTransaction) -> Result<bool> {
        let Some((fee_per_cost, rules)) = self.rate else {
            return Ok(true);
        };

        let coin_spends = transaction.priced_coin_spends();

        if coin_spends.is_empty() {
            return Ok(true);
        }

        let (cost, paid) = rules.price(&coin_spends)?;
        let needed = fee_for_cost(cost, fee_per_cost);

        if paid >= needed {
            return Ok(true);
        }

        self.attempts += 1;

        if self.attempts >= MAX_FEE_ATTEMPTS {
            return Err(DriverError::FeeDidNotConverge.into());
        }

        self.fee = self
            .fee
            .checked_add(needed - paid)
            .ok_or_else(|| Error::InvalidAmount("the fee overflows".to_string()))?;

        Ok(false)
    }
}

/// Builds a transaction with `$build`, which can use the fee to pay as `$fee`, and evaluates to
/// it.
///
/// Without a fee rate, it's built once with `$base_fee`. With one, `$base_fee` is the least it
/// pays: the transaction is rebuilt with a higher fee until the fee it pays, including any fee
/// already paid by spends it doesn't add (such as the maker's side of an offer), covers its cost
/// at `$fee_per_cost` mojos per cost. Adding a coin to pay the fee raises the cost, so this
/// normally takes two or three attempts. The fee only grows between attempts, and the loop
/// stops at the first attempt whose fee covers its cost. Since a higher fee can change which
/// coins are selected, that fee can be a little more than the final coins need.
///
/// It's a macro rather than a function taking an async closure, since the futures of async
/// closures that borrow aren't `Send`, which the RPC server needs.
macro_rules! build_with_fee {
    ($sage:expr, $base_fee:expr, $fee_per_cost:expr, |$fee:ident| $build:expr) => {{
        let mut pricer = $sage.fee_pricer($base_fee, $fee_per_cost).await?;

        loop {
            let $fee = pricer.fee();
            let transaction = $build;

            if pricer.covers(&transaction)? {
                break transaction;
            }
        }
    }};
}

pub(crate) use build_with_fee;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fee_for_cost_rounds_up() {
        assert_eq!(fee_for_cost(0, 5.0), 0);
        assert_eq!(fee_for_cost(1_000, 5.0), 5_000);
        assert_eq!(fee_for_cost(3, 0.5), 2);
        assert_eq!(fee_for_cost(1_000, 0.0), 0);
    }
}
