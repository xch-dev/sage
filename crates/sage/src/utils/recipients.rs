//! The recipient pipeline, shared by every endpoint which takes a recipient.
//!
//! A recipient is either an address, or an XCHandles handle written as `@handle`. Handles
//! are resolved through the verified resolver before the transaction is created, and:
//!
//! - A handle which resolves to a different address than the one it changed from, or than
//!   the one last sent to, is rejected until the new address is confirmed.
//! - A handle which was confirmed as resolving to one address, but resolves to another by
//!   the time the request is made, is rejected.
//! - A transaction which isn't submitted straight away remembers which handles it pays, and
//!   they're resolved again right before it's submitted. If any of them changed in the
//!   meantime, it isn't submitted.
//! - Once a transaction is submitted, the addresses its handles resolved to are recorded as
//!   the addresses last sent to.
//!
//! Requests without any handles behave exactly as they did before handles were supported.

use std::collections::{HashMap, HashSet};

use chia_wallet_sdk::{prelude::*, utils::Address};
use futures_util::future::join_all;
use indexmap::{IndexMap, IndexSet};
use sage_api::{HandleConfirmation, RecipientError, RecipientErrorKind};
use sage_wallet::{Handle, HandleResolution, XchandlesError};
use tracing::warn;

use crate::{ConfirmationInfo, Error, Result, Sage};

/// How many transactions which haven't been submitted yet to remember the handle
/// recipients of. The oldest are forgotten first, and are then submitted without being
/// checked again, as if they had been signed elsewhere.
const MAX_PENDING_TRANSACTIONS: usize = 64;

/// A handle recipient, and the verified address it was resolved to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandleSend {
    pub handle: Handle,
    pub puzzle_hash: Bytes32,
}

/// The handle recipients of transactions which were created but haven't been submitted,
/// keyed by [`pending_key`].
#[derive(Debug, Default)]
pub struct PendingHandleSends(IndexMap<Bytes32, Vec<HandleSend>>);

impl PendingHandleSends {
    pub fn insert(&mut self, key: Bytes32, sends: Vec<HandleSend>) {
        self.0.shift_remove(&key);
        self.0.insert(key, sends);

        while self.0.len() > MAX_PENDING_TRANSACTIONS {
            self.0.shift_remove_index(0);
        }
    }

    pub fn remove(&mut self, key: Bytes32) -> Option<Vec<HandleSend>> {
        self.0.shift_remove(&key)
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }
}

/// Recipients which were resolved ahead of time, keyed by the recipient as it was given.
#[derive(Debug, Default)]
pub struct ResolvedRecipients(HashMap<String, Bytes32>);

impl ResolvedRecipients {
    pub fn get(&self, input: &str) -> Result<Bytes32> {
        self.0.get(input).copied().ok_or_else(|| {
            Error::Recipients(vec![RecipientError {
                recipient: input.to_string(),
                kind: RecipientErrorKind::Invalid,
                reason: format!("{input} was not resolved"),
                handle: None,
                address: None,
                previous_address: None,
            }])
        })
    }
}

/// Identifies a transaction by its coin spends, which signing doesn't change.
pub fn pending_key(coin_spends: &[CoinSpend]) -> Bytes32 {
    let mut coin_spends = coin_spends.to_vec();
    coin_spends.sort_by_key(|coin_spend| coin_spend.coin.coin_id());
    SpendBundle::new(coin_spends, Signature::default()).name()
}

/// The address a resolution changed from without the change being confirmed, if any.
///
/// A confirmed address must match exactly. Otherwise, the handle must resolve to the address
/// it was last verified as, unless that change was acknowledged, and to the address it was
/// last sent to.
pub(crate) fn unconfirmed_change(
    resolution: &HandleResolution,
    confirmed: Option<Bytes32>,
) -> Option<Bytes32> {
    let current = resolution.row.resolved_puzzle_hash;

    if let Some(confirmed) = confirmed {
        return (confirmed != current).then_some(confirmed);
    }

    resolution.changed_from().or(resolution
        .row
        .last_sent_puzzle_hash
        .filter(|&last_sent| last_sent != current))
}

fn recipient_error_kind(error: &XchandlesError) -> RecipientErrorKind {
    match error {
        XchandlesError::InvalidHandle(..) => RecipientErrorKind::Invalid,
        XchandlesError::UnsupportedNetwork => RecipientErrorKind::UnsupportedNetwork,
        error if error.is_not_registered() => RecipientErrorKind::NotRegistered,
        error if error.is_unavailable() => RecipientErrorKind::Unavailable,
        _ => RecipientErrorKind::Unverified,
    }
}

impl Sage {
    /// Resolves a single recipient. See [`Sage::resolve_recipients`].
    pub(crate) async fn resolve_recipient(
        &self,
        input: &str,
        confirmations: &[HandleConfirmation],
        info: &mut ConfirmationInfo,
    ) -> Result<Bytes32> {
        let mut puzzle_hashes = self
            .resolve_recipients(&[input], confirmations, info)
            .await?;
        Ok(puzzle_hashes.remove(0))
    }

    /// Resolves recipients to puzzle hashes, in the same order. Handles are verified and
    /// checked for changes, and added to the confirmation info so that they're shown in the
    /// summary and recorded when the transaction is submitted.
    ///
    /// If none of the recipients are handles, the first invalid address is returned as its
    /// own error, as before. Otherwise every recipient which can't be used is reported in a
    /// single [`Error::Recipients`].
    pub(crate) async fn resolve_recipients(
        &self,
        inputs: &[&str],
        confirmations: &[HandleConfirmation],
        info: &mut ConfirmationInfo,
    ) -> Result<Vec<Bytes32>> {
        // Surrounding whitespace is ignored, as it is when recipients are previewed.
        let inputs: Vec<&str> = inputs.iter().map(|input| input.trim()).collect();
        let inputs = inputs.as_slice();

        if !inputs.iter().any(|input| input.starts_with('@')) {
            return inputs
                .iter()
                .map(|input| self.parse_address((*input).to_string()))
                .collect();
        }

        let confirmed = self.parse_handle_confirmations(confirmations)?;

        let handles: IndexSet<Handle> = inputs
            .iter()
            .filter_map(|input| Handle::from_input(input)?.ok())
            .collect();

        let lookups = join_all(
            handles
                .iter()
                .map(|handle| self.resolve_handle_mapping(handle)),
        );
        let mut resolutions = HashMap::new();

        for (handle, result) in handles.iter().zip(lookups.await) {
            let result = match result {
                Ok(resolution) => Ok(resolution),
                Err(Error::Xchandles(error)) if !matches!(error, XchandlesError::Database(..)) => {
                    Err(error)
                }
                Err(error) => return Err(error),
            };
            resolutions.insert(handle.clone(), result);
        }

        let mut puzzle_hashes = Vec::with_capacity(inputs.len());
        let mut errors = Vec::new();
        let mut reported = HashSet::new();

        for &input in inputs {
            match self.check_recipient(input, &confirmed, &resolutions)? {
                Ok(puzzle_hash) => puzzle_hashes.push(puzzle_hash),
                Err(error) => {
                    if reported.insert(input) {
                        errors.push(error);
                    }
                }
            }
        }

        if !errors.is_empty() {
            return Err(Error::Recipients(errors));
        }

        for handle in handles {
            let Some(Ok(resolution)) = resolutions.get(&handle) else {
                continue;
            };

            if info.handle_sends.iter().any(|send| send.handle == handle) {
                continue;
            }

            info.handles.push(self.handle_record(resolution)?);
            info.handle_sends.push(HandleSend {
                handle,
                puzzle_hash: resolution.row.resolved_puzzle_hash,
            });
        }

        Ok(puzzle_hashes)
    }

    /// Resolves recipients which are used in different parts of a request ahead of time, so
    /// that every recipient which can't be used is reported at once.
    pub(crate) async fn resolve_recipient_map(
        &self,
        inputs: &[&str],
        confirmations: &[HandleConfirmation],
        info: &mut ConfirmationInfo,
    ) -> Result<ResolvedRecipients> {
        let inputs: Vec<&str> = inputs
            .iter()
            .copied()
            .collect::<IndexSet<_>>()
            .into_iter()
            .collect();
        let puzzle_hashes = self
            .resolve_recipients(&inputs, confirmations, info)
            .await?;

        Ok(ResolvedRecipients(
            inputs
                .into_iter()
                .map(str::to_string)
                .zip(puzzle_hashes)
                .collect(),
        ))
    }

    fn parse_handle_confirmations(
        &self,
        confirmations: &[HandleConfirmation],
    ) -> Result<HashMap<Handle, Bytes32>> {
        let mut confirmed = HashMap::new();

        for confirmation in confirmations {
            let handle = Handle::parse(confirmation.handle.trim()).map_err(XchandlesError::from)?;
            let puzzle_hash = self.parse_address(confirmation.address.clone())?;

            if confirmed
                .insert(handle.clone(), puzzle_hash)
                .is_some_and(|existing| existing != puzzle_hash)
            {
                return Err(Error::Recipients(vec![RecipientError {
                    recipient: handle.to_string(),
                    kind: RecipientErrorKind::Invalid,
                    reason: format!("{handle} was confirmed with more than one address"),
                    handle: Some(handle.as_str().to_string()),
                    address: None,
                    previous_address: None,
                }]));
            }
        }

        Ok(confirmed)
    }

    fn check_recipient(
        &self,
        input: &str,
        confirmed: &HashMap<Handle, Bytes32>,
        resolutions: &HashMap<Handle, std::result::Result<HandleResolution, XchandlesError>>,
    ) -> Result<std::result::Result<Bytes32, RecipientError>> {
        let invalid = |reason: String, handle: Option<&Handle>| RecipientError {
            recipient: input.to_string(),
            kind: RecipientErrorKind::Invalid,
            reason,
            handle: handle.map(|handle| handle.as_str().to_string()),
            address: None,
            previous_address: None,
        };

        let handle = match Handle::from_input(input) {
            None => {
                return Ok(self.parse_address(input.to_string()).map_err(|error| {
                    invalid(format!("{input} is not a valid address: {error}"), None)
                }));
            }
            Some(Err(error)) => {
                return Ok(Err(invalid(
                    format!("{input} is not a valid handle: {error}"),
                    None,
                )));
            }
            Some(Ok(handle)) => handle,
        };

        let resolution = match resolutions.get(&handle) {
            Some(Ok(resolution)) => resolution,
            Some(Err(error)) => {
                return Ok(Err(RecipientError {
                    recipient: input.to_string(),
                    kind: recipient_error_kind(error),
                    reason: format!("{handle} could not be resolved: {error}"),
                    handle: Some(handle.as_str().to_string()),
                    address: None,
                    previous_address: None,
                }));
            }
            None => return Err(Error::Xchandles(XchandlesError::ResolutionUnavailable)),
        };

        let current = resolution.row.resolved_puzzle_hash;

        let Some(previous) = unconfirmed_change(resolution, confirmed.get(&handle).copied()) else {
            return Ok(Ok(current));
        };

        Ok(Err(self.changed_recipient_error(
            input, &handle, previous, current,
        )?))
    }

    fn changed_recipient_error(
        &self,
        input: &str,
        handle: &Handle,
        previous: Bytes32,
        current: Bytes32,
    ) -> Result<RecipientError> {
        let prefix = self.network().prefix();
        let previous = Address::new(previous, prefix.clone()).encode()?;
        let current = Address::new(current, prefix).encode()?;

        Ok(RecipientError {
            recipient: input.to_string(),
            kind: RecipientErrorKind::Changed,
            reason: format!(
                "{handle} now resolves to {current}, but resolved to {previous} before. \
                It may have changed hands. Confirm the new address to send to it."
            ),
            handle: Some(handle.as_str().to_string()),
            address: Some(current),
            previous_address: Some(previous),
        })
    }

    /// Remembers the handle recipients of a transaction which wasn't submitted, so that
    /// they can be checked again when it is.
    pub(crate) fn remember_handle_sends(&self, coin_spends: &[CoinSpend], sends: Vec<HandleSend>) {
        if sends.is_empty() {
            return;
        }

        self.pending_handle_sends
            .lock()
            .expect("pending handle sends lock poisoned")
            .insert(pending_key(coin_spends), sends);
    }

    /// Resolves the handle recipients of a transaction again right before it's submitted,
    /// so that a handle which changed since the transaction was reviewed can't redirect it.
    ///
    /// Returns the handle sends to record once the transaction has been submitted.
    pub(crate) async fn recheck_handle_sends(
        &self,
        coin_spends: &[CoinSpend],
    ) -> Result<Vec<HandleSend>> {
        let key = pending_key(coin_spends);

        let Some(sends) = self
            .pending_handle_sends
            .lock()
            .expect("pending handle sends lock poisoned")
            .remove(key)
        else {
            return Ok(Vec::new());
        };

        let result = self.recheck_sends(&sends).await;

        if result.is_err() {
            // The transaction may be submitted again, for example once the handle can be
            // verified, so it has to be checked again then too.
            self.pending_handle_sends
                .lock()
                .expect("pending handle sends lock poisoned")
                .insert(key, sends.clone());
        }

        result.map(|()| sends)
    }

    async fn recheck_sends(&self, sends: &[HandleSend]) -> Result<()> {
        let lookups = join_all(
            sends
                .iter()
                .map(|send| self.resolve_handle_mapping(&send.handle)),
        );
        let mut errors = Vec::new();

        for (send, result) in sends.iter().zip(lookups.await) {
            let input = send.handle.to_string();

            match result {
                Ok(resolution) => {
                    let current = resolution.row.resolved_puzzle_hash;

                    if current != send.puzzle_hash {
                        let mut error = self.changed_recipient_error(
                            &input,
                            &send.handle,
                            send.puzzle_hash,
                            current,
                        )?;

                        // The transaction pays the old address, so the new one can only be
                        // confirmed by creating it again.
                        error.reason = format!(
                            "{} changed after this transaction was created. It now resolves \
                            to {}, but the transaction sends to {}. Create the transaction \
                            again to send to the new address.",
                            send.handle,
                            error.address.as_deref().unwrap_or_default(),
                            error.previous_address.as_deref().unwrap_or_default(),
                        );

                        errors.push(error);
                    }
                }
                Err(Error::Xchandles(error)) if !matches!(error, XchandlesError::Database(..)) => {
                    errors.push(RecipientError {
                        recipient: input,
                        kind: recipient_error_kind(&error),
                        reason: format!(
                            "{} could not be verified again before submitting: {error}",
                            send.handle
                        ),
                        handle: Some(send.handle.as_str().to_string()),
                        address: None,
                        previous_address: None,
                    });
                }
                Err(error) => return Err(error),
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(Error::Recipients(errors))
        }
    }

    /// Records the addresses handles resolved to as the addresses last sent to, once the
    /// transaction has been submitted. This also acknowledges any change, since it was
    /// confirmed before the transaction was created.
    ///
    /// The transaction has already been submitted, so failures are only logged.
    pub(crate) async fn record_handle_sends(&self, sends: &[HandleSend]) {
        if sends.is_empty() {
            return;
        }

        let Some(config) = self.network().xchandles() else {
            return;
        };

        let Ok(wallet) = self.wallet() else {
            return;
        };

        for send in sends {
            let handle = send.handle.as_str();
            let registry_launcher_id = config.registry_launcher_id;

            let result = async {
                let mut tx = wallet.db.tx().await?;

                if let Some(row) = tx.xchandle(registry_launcher_id, handle).await?
                    && row.resolved_puzzle_hash == send.puzzle_hash
                {
                    tx.acknowledge_xchandle_change(registry_launcher_id, handle)
                        .await?;
                }

                tx.set_xchandle_last_sent(registry_launcher_id, handle, send.puzzle_hash)
                    .await?;
                tx.commit().await
            }
            .await;

            if let Err(error) = result {
                warn!(
                    "Failed to record the address sent to for {}: {error}",
                    send.handle
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use sage_database::{XchandleRow, XchandleStatus};

    use super::*;

    fn resolution(
        resolved: Bytes32,
        previous: Option<Bytes32>,
        last_sent: Option<Bytes32>,
    ) -> HandleResolution {
        HandleResolution {
            row: XchandleRow {
                registry_launcher_id: Bytes32::default(),
                handle: "alice".to_string(),
                handle_hash: Bytes32::default(),
                owner_launcher_id: Bytes32::default(),
                resolved_launcher_id: Bytes32::default(),
                resolved_puzzle_hash: resolved,
                expiration: u64::MAX,
                slot_coin_id: Bytes32::default(),
                slot_counter: 0,
                slot_left_handle_hash: Bytes32::default(),
                slot_right_handle_hash: Bytes32::default(),
                slot_parent_id: Bytes32::default(),
                slot_parent_parent_id: Bytes32::default(),
                slot_parent_inner_puzzle_hash: Bytes32::default(),
                slot_confirmation_height: 0,
                resolved_coin_id: Bytes32::default(),
                resolved_confirmation_height: 0,
                indexed_height: 0,
                verified_height: 0,
                verified_timestamp: 0,
                last_verified_at: 0,
                status: XchandleStatus::Verified,
                status_reason: None,
                previous_resolved_launcher_id: previous.map(|_| Bytes32::default()),
                previous_resolved_puzzle_hash: previous,
                last_sent_puzzle_hash: last_sent,
            },
            cached: false,
        }
    }

    #[test]
    fn test_unconfirmed_change() {
        let a = Bytes32::new([1; 32]);
        let b = Bytes32::new([2; 32]);

        // A handle used for the first time, or sent to before at the same address.
        assert_eq!(unconfirmed_change(&resolution(a, None, None), None), None);
        assert_eq!(
            unconfirmed_change(&resolution(a, None, Some(a)), None),
            None
        );

        // The resolved address changed since it was last verified, or last sent to.
        assert_eq!(
            unconfirmed_change(&resolution(b, Some(a), None), None),
            Some(a)
        );
        assert_eq!(
            unconfirmed_change(&resolution(b, None, Some(a)), None),
            Some(a)
        );
        assert_eq!(
            unconfirmed_change(&resolution(b, Some(a), Some(a)), None),
            Some(a)
        );

        // Confirming the new address acknowledges the change.
        assert_eq!(
            unconfirmed_change(&resolution(b, Some(a), None), Some(b)),
            None
        );
        assert_eq!(
            unconfirmed_change(&resolution(b, None, Some(a)), Some(b)),
            None
        );

        // A confirmation of the old address doesn't match the new one.
        assert_eq!(
            unconfirmed_change(&resolution(b, Some(a), None), Some(a)),
            Some(a)
        );
        assert_eq!(
            unconfirmed_change(&resolution(b, None, None), Some(a)),
            Some(a)
        );
    }

    #[test]
    fn test_pending_key_ignores_order() {
        let coin_spend = |amount| {
            CoinSpend::new(
                Coin::new(Bytes32::default(), Bytes32::default(), amount),
                Program::default(),
                Program::default(),
            )
        };

        let a = coin_spend(1);
        let b = coin_spend(2);

        assert_eq!(
            pending_key(&[a.clone(), b.clone()]),
            pending_key(&[b.clone(), a.clone()])
        );
        assert_ne!(pending_key(&[a]), pending_key(&[b]));
    }

    #[test]
    fn test_pending_handle_sends_are_bounded() {
        let mut pending = PendingHandleSends::default();

        for i in 0..=MAX_PENDING_TRANSACTIONS {
            pending.insert(Bytes32::new([u8::try_from(i).unwrap(); 32]), Vec::new());
        }

        assert_eq!(pending.remove(Bytes32::new([0; 32])), None);
        assert!(pending.remove(Bytes32::new([1; 32])).is_some());
    }
}
