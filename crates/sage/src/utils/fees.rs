use std::{borrow::Cow, time::Duration};

use chia_wallet_sdk::{
    chia::consensus::{
        consensus_constants::ConsensusConstants, owned_conditions::OwnedSpendBundleConditions,
    },
    coinset::{ChiaRpcClient, GetFeeEstimateResponse},
    driver::{MIN_FEE_PER_COST, spend_bundle_conditions},
    prelude::*,
};
use sage_api::{FeeTier, FeeTierName};
use sage_config::InheritedNetwork;
use sage_wallet::{PreSplitOffer, TakenOffer, coinset_client};
use tokio::time::timeout;
use tracing::warn;

use crate::{Error, Result, Sage};

/// The most times a transaction is rebuilt to find a fee that covers its cost. Each rebuild
/// raises the fee by what the previous attempt was missing, and the fee never goes down, so it
/// normally settles within two or three attempts.
const MAX_FEE_ATTEMPTS: usize = 8;

/// The fee rate, in mojos per cost, that a transaction is priced at when the wallet calculates
/// its fee. It's the minimum at which farmers that enforce a minimum block fee include it in a
/// block, and a full mempool admits it.
pub const RECOMMENDED_FEE_PER_COST: u64 = MIN_FEE_PER_COST;

/// The fee tiers that Coinset estimates rates for, fastest first, with the time in seconds
/// that a transaction paying each is expected to be confirmed within.
const FEE_TIERS: [(FeeTierName, u64); 3] = [
    (FeeTierName::Fast, 60),
    (FeeTierName::Standard, 5 * 60),
    (FeeTierName::Economy, 60 * 60),
];

/// The target times of [`FEE_TIERS`], in the same order.
const FEE_TIER_TARGET_TIMES: [u64; FEE_TIERS.len()] = {
    let mut target_times = [0; FEE_TIERS.len()];
    let mut i = 0;
    while i < FEE_TIERS.len() {
        target_times[i] = FEE_TIERS[i].1;
        i += 1;
    }
    target_times
};

/// The cost that fee estimates are requested for, which is about that of a typical
/// transaction. An estimate is a fee for this cost, so it's divided by it to get a rate.
const FEE_ESTIMATE_COST: u64 = 50_000_000;

/// How long a request for fee estimates can take before the estimates are left out.
const FEE_ESTIMATE_TIMEOUT: Duration = Duration::from_secs(5);

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
        // fork, as the SDK's `spend_bundle_cost` prices it. The fee is the same on both sides.
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
    /// transaction, such as one side of an offer. It's the cost the fee is priced against.
    pub(crate) fn cost(&self, coin_spends: &[CoinSpend]) -> Option<u64> {
        self.price(coin_spends).ok().map(|(cost, _)| cost)
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

/// What's left of `amount` once `fee` is taken out of it, for a transaction whose fee is
/// deducted from the amount it sends.
pub(crate) fn amount_after_fee(amount: u64, fee: u64) -> Result<u64> {
    amount
        .checked_sub(fee)
        .filter(|&amount| amount > 0)
        .ok_or_else(|| {
            Error::InvalidAmount(format!(
                "the amount of {amount} mojos doesn't cover the fee of {fee} mojos"
            ))
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

/// The fee tiers for Coinset's estimates, or none if it isn't synced or didn't estimate every
/// tier.
///
/// Each rate is at least the minimum fee rate, so a transaction paying it is included in blocks
/// by farmers that enforce a minimum fee, and a faster tier's rate is never below a slower
/// tier's. Rates are rounded up to a hundredth of a
/// mojo per cost, so they're shown as they're paid.
fn fee_tiers(response: &GetFeeEstimateResponse) -> Vec<FeeTier> {
    if !response.success
        || response.full_node_synced == Some(false)
        || response.target_times != FEE_TIER_TARGET_TIMES
        || response.estimates.len() != FEE_TIERS.len()
    {
        return Vec::new();
    }

    // `current_fee_rate` isn't used as a floor: it's the estimate for the next block, the
    // same as the fastest tier's, rather than the least the mempool admits.
    #[allow(clippy::cast_precision_loss)]
    let minimum = MIN_FEE_PER_COST as f64;

    // From the slowest tier to the fastest, so that each is at least the one before it.
    let mut floor = minimum;

    let mut tiers: Vec<FeeTier> = FEE_TIERS
        .iter()
        .zip(&response.estimates)
        .rev()
        .map(|(&(name, target_time), &estimate)| {
            // Rounded up in whole mojos rather than as a float, where a rate of exactly 5.11
            // can be 511.00000000000006 hundredths and round up to 5.12.
            let hundredths = estimate.div_ceil(FEE_ESTIMATE_COST / 100);
            #[allow(clippy::cast_precision_loss)]
            let fee_per_cost = (hundredths as f64 / 100.0).max(floor);
            floor = fee_per_cost;

            FeeTier {
                name,
                target_time,
                fee_per_cost,
            }
        })
        .collect();

    tiers.reverse();
    tiers
}

impl Sage {
    /// The fee tiers estimated by the network's Coinset API, or by [`Sage::fee_estimator`] if
    /// it's set. There are none for a network that Coinset doesn't serve, or if the estimate
    /// can't be fetched. Tests never use the real Coinset API.
    pub(crate) async fn fee_tiers(&self) -> Vec<FeeTier> {
        let client = match &self.fee_estimator {
            Some(client) => Some(client.clone()),
            None if self.test => None,
            None => coinset_client(self.network()),
        };

        let Some(client) = client else {
            return Vec::new();
        };

        let request = client.get_fee_estimate(FEE_ESTIMATE_COST, FEE_TIER_TARGET_TIMES.to_vec());

        match timeout(FEE_ESTIMATE_TIMEOUT, request).await {
            Ok(Ok(response)) => {
                let tiers = fee_tiers(&response);

                if tiers.is_empty() {
                    warn!(
                        "Coinset didn't estimate fees: {}",
                        response
                            .error
                            .as_deref()
                            .unwrap_or("not synced, or an unexpected response")
                    );
                }

                tiers
            }
            Ok(Err(error)) => {
                warn!("Couldn't fetch fee estimates from Coinset: {error}");
                Vec::new()
            }
            Err(_) => {
                warn!("Timed out fetching fee estimates from Coinset");
                Vec::new()
            }
        }
    }

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
    fn test_amount_after_fee() {
        assert_eq!(amount_after_fee(1_000, 300).ok(), Some(700));
        assert!(amount_after_fee(1_000, 1_000).is_err());
        assert!(amount_after_fee(1_000, 1_001).is_err());
    }

    fn estimate(estimates: [u64; 3], current_fee_rate: Option<f64>) -> GetFeeEstimateResponse {
        GetFeeEstimateResponse {
            estimates: estimates.to_vec(),
            target_times: FEE_TIER_TARGET_TIMES.to_vec(),
            current_fee_rate,
            mempool_size: None,
            mempool_max_size: None,
            full_node_synced: Some(true),
            last_tx_block_height: None,
            error: None,
            success: true,
        }
    }

    fn rates(tiers: &[FeeTier]) -> Vec<f64> {
        tiers.iter().map(|tier| tier.fee_per_cost).collect()
    }

    #[test]
    fn test_fee_tiers_use_the_estimates() {
        let tiers = fee_tiers(&estimate([1_000_000_000, 500_000_000, 300_000_000], None));

        assert_eq!(
            tiers.iter().map(|tier| tier.name).collect::<Vec<_>>(),
            [
                FeeTierName::Fast,
                FeeTierName::Standard,
                FeeTierName::Economy
            ]
        );
        assert_eq!(
            tiers
                .iter()
                .map(|tier| tier.target_time)
                .collect::<Vec<_>>(),
            [60, 300, 3600]
        );
        assert_eq!(rates(&tiers), [20.0, 10.0, 6.0]);
    }

    #[test]
    fn test_fee_tiers_without_a_sync_status() {
        // A response that doesn't say whether the node is synced is still used.
        let mut response = estimate([1_000_000_000, 500_000_000, 300_000_000], None);
        response.full_node_synced = None;
        assert_eq!(rates(&fee_tiers(&response)), [20.0, 10.0, 6.0]);
    }

    #[test]
    fn test_fee_tiers_are_at_least_the_minimum() {
        let tiers = fee_tiers(&estimate([100_000_000, 0, 0], Some(2.0)));
        assert_eq!(rates(&tiers), [5.0, 5.0, 5.0]);

        // As Coinset reports it, with the current rate the same as the fastest tier's, which
        // doesn't raise the slower tiers.
        let tiers = fee_tiers(&estimate([2_407_289_013, 69_315_967, 0], Some(48.1458)));
        assert_eq!(rates(&tiers), [48.15, 5.0, 5.0]);
    }

    #[test]
    fn test_fee_tiers_are_ordered_and_rounded_up() {
        // A slower tier's estimate above a faster one's raises the faster one.
        let tiers = fee_tiers(&estimate([400_000_000, 300_000_001, 600_000_000], None));
        assert_eq!(rates(&tiers), [12.0, 12.0, 12.0]);

        let tiers = fee_tiers(&estimate([300_000_001, 0, 0], None));
        assert_eq!(rates(&tiers), [6.01, 5.0, 5.0]);

        // A rate that's already a whole hundredth isn't rounded up past it.
        let tiers = fee_tiers(&estimate([402_500_000, 255_500_000, 0], None));
        assert_eq!(rates(&tiers), [8.05, 5.11, 5.0]);
    }

    #[test]
    fn test_fee_tiers_without_an_estimate() {
        let mut response = estimate([0, 0, 0], None);
        response.full_node_synced = Some(false);
        assert!(fee_tiers(&response).is_empty());

        let mut response = estimate([0, 0, 0], None);
        response.success = false;
        assert!(fee_tiers(&response).is_empty());

        let mut response = estimate([0, 0, 0], None);
        response.estimates.pop();
        assert!(fee_tiers(&response).is_empty());

        let mut response = estimate([0, 0, 0], None);
        response.target_times.reverse();
        assert!(fee_tiers(&response).is_empty());
    }

    #[test]
    fn test_fee_for_cost_rounds_up() {
        assert_eq!(fee_for_cost(0, 5.0), 0);
        assert_eq!(fee_for_cost(1_000, 5.0), 5_000);
        assert_eq!(fee_for_cost(3, 0.5), 2);
        assert_eq!(fee_for_cost(1_000, 0.0), 0);
    }
}
