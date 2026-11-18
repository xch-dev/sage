//! A stand-in for Coinset's `get_fee_estimate`, so that the UI's fee speeds have rates to show.
//! The simulator has no mempool to estimate from, so the rates are made up, and can be changed
//! with `POST /fee-estimates`.

use std::sync::{Arc, PoisonError, RwLock};

use axum::{
    Json,
    extract::{State, rejection::JsonRejection},
    response::{IntoResponse, Response},
};
use chia_wallet_sdk::coinset::GetFeeEstimateResponse;
use serde::{Deserialize, Serialize};

use crate::{BridgeError, bad_request};

/// The rates, in mojos per cost, that the stand-in estimates for each fee speed.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FeeEstimates {
    /// For a transaction to be confirmed within about a minute
    pub fast: f64,
    /// Within about five minutes
    pub standard: f64,
    /// Within about an hour
    pub economy: f64,
}

impl Default for FeeEstimates {
    /// A busy mempool, so that each speed has its own rate. Economy is below the minimum fee
    /// rate, to show it being raised to the minimum.
    fn default() -> Self {
        Self {
            fast: 20.0,
            standard: 10.0,
            economy: 2.0,
        }
    }
}

impl FeeEstimates {
    /// The rate for a transaction to be confirmed within `target_time` seconds.
    fn rate(&self, target_time: u64) -> f64 {
        match target_time {
            0..=60 => self.fast,
            61..=300 => self.standard,
            _ => self.economy,
        }
    }
}

/// The estimates the stand-in gives, shared by every wallet since they're on the same chain.
/// `None` means it has none, as when Coinset fails, so every speed pays the recommended rate.
#[derive(Debug, Clone, Default)]
pub struct FeeEstimatesState(Arc<RwLock<Option<FeeEstimates>>>);

impl FeeEstimatesState {
    pub fn new(estimates: Option<FeeEstimates>) -> Self {
        Self(Arc::new(RwLock::new(estimates)))
    }

    fn get(&self) -> Option<FeeEstimates> {
        *self.0.read().unwrap_or_else(PoisonError::into_inner)
    }

    fn set(&self, estimates: Option<FeeEstimates>) {
        *self.0.write().unwrap_or_else(PoisonError::into_inner) = estimates;
    }
}

#[derive(Debug, Deserialize)]
pub struct FeeEstimateRequest {
    cost: u64,
    target_times: Vec<u64>,
}

/// Answers `get_fee_estimate` as Coinset does, with the fee for `cost` at each target time's rate.
pub async fn get_fee_estimate(
    State(state): State<FeeEstimatesState>,
    Json(request): Json<FeeEstimateRequest>,
) -> Json<GetFeeEstimateResponse> {
    let estimates = state.get();

    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss
    )]
    let fees = estimates.map(|estimates| {
        request
            .target_times
            .iter()
            .map(|&target_time| (estimates.rate(target_time) * request.cost as f64).ceil() as u64)
            .collect()
    });

    Json(GetFeeEstimateResponse {
        success: fees.is_some(),
        error: fees
            .is_none()
            .then(|| "The sandbox's fee estimates are turned off".to_string()),
        estimates: fees.unwrap_or_default(),
        target_times: request.target_times,
        current_fee_rate: None,
        mempool_size: None,
        mempool_max_size: None,
        full_node_synced: Some(true),
        last_tx_block_height: None,
    })
}

/// Returns the estimates the stand-in gives, or `null` if they're turned off.
pub async fn fee_estimates(State(state): State<FeeEstimatesState>) -> Json<Option<FeeEstimates>> {
    Json(state.get())
}

/// Replaces the estimates the stand-in gives, or turns them off with `null`.
pub async fn set_fee_estimates(
    State(state): State<FeeEstimatesState>,
    body: Result<Json<Option<FeeEstimates>>, JsonRejection>,
) -> Response {
    let estimates = match body {
        Ok(Json(estimates)) => estimates,
        Err(rejection) => return bad_request(BridgeError::api(rejection.body_text())),
    };

    if let Some(estimates) = estimates
        && [estimates.fast, estimates.standard, estimates.economy]
            .iter()
            .any(|rate| !rate.is_finite() || *rate < 0.0)
    {
        return bad_request(BridgeError::api(
            "Fee estimates must be rates of at least 0 mojos per cost",
        ));
    }

    state.set(estimates);
    Json(estimates).into_response()
}
