use chia_wallet_sdk::prelude::*;
use tracing::{info, warn};

use crate::{WalletError, WalletPeer};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    /// The node's ack status: 1 (SUCCESS, in the mempool) or 2 (PENDING, held by the node).
    Pending(u8),
    Failed(u8, Option<String>),
    Unknown,
}

pub async fn submit_to_peers(
    peers: &[WalletPeer],
    spend_bundle: SpendBundle,
) -> Result<Status, WalletError> {
    let transaction_id = spend_bundle.name();

    info!("Checking transaction id {}", transaction_id);

    let mut results = Vec::with_capacity(peers.len());

    for peer in peers {
        let ip = peer.socket_addr().ip();
        let result = submit_transaction(peer, spend_bundle.clone()).await?;

        if matches!(result, Status::Failed(..)) {
            info!(
                "Transaction {transaction_id} failed according to peer {ip}, but will check other peers"
            );
        }

        results.push(result);
    }

    Ok(combine_peer_results(results))
}

/// Pending if any peer accepted the transaction, preferring SUCCESS (1) over PENDING (2).
/// Otherwise the last failure, or unknown if no peer answered.
fn combine_peer_results(results: impl IntoIterator<Item = Status>) -> Status {
    let mut pending: Option<u8> = None;
    let mut failure = None;

    for result in results {
        match result {
            Status::Pending(status) => {
                pending = Some(pending.map_or(status, |current| current.min(status)));
            }
            Status::Failed(status, error) => failure = Some((status, error)),
            Status::Unknown => {}
        }
    }

    if let Some(status) = pending {
        Status::Pending(status)
    } else if let Some((status, error)) = failure {
        Status::Failed(status, error)
    } else {
        Status::Unknown
    }
}

pub async fn submit_transaction(
    peer: &WalletPeer,
    spend_bundle: SpendBundle,
) -> Result<Status, WalletError> {
    info!("Submitting transaction to {}", peer.socket_addr());

    let ack = match peer.send_transaction(spend_bundle.clone()).await {
        Ok(ack) => ack,
        Err(err) => {
            warn!(
                "Send transaction failed for {}: {}",
                peer.socket_addr(),
                err
            );
            return Ok(Status::Unknown);
        }
    };

    info!(
        "Transaction response received from {} with ack {:?}",
        peer.socket_addr(),
        ack
    );

    Ok(ack_status(ack.status, ack.error))
}

fn ack_status(status: u8, error: Option<String>) -> Status {
    // 1 is SUCCESS (in the mempool) and 2 is PENDING (held by the node, for example
    // until a timelock passes). Either way the transaction can still be confirmed.
    if status == 1 || status == 2 {
        Status::Pending(status)
    } else {
        Status::Failed(status, error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pending_acks_are_not_failures() {
        assert_eq!(ack_status(1, None), Status::Pending(1));
        assert_eq!(
            ack_status(2, Some("ASSERT_HEIGHT_ABSOLUTE_FAILED".to_string())),
            Status::Pending(2)
        );
        assert_eq!(
            ack_status(3, Some("DOUBLE_SPEND".to_string())),
            Status::Failed(3, Some("DOUBLE_SPEND".to_string()))
        );
    }

    #[test]
    fn peer_results_prefer_acceptance_then_success() {
        let failed = || Status::Failed(3, Some("DOUBLE_SPEND".to_string()));

        assert_eq!(
            combine_peer_results([failed(), Status::Pending(2), Status::Unknown]),
            Status::Pending(2)
        );
        assert_eq!(
            combine_peer_results([Status::Pending(2), Status::Pending(1), Status::Pending(2)]),
            Status::Pending(1)
        );
        assert_eq!(
            combine_peer_results([Status::Unknown, failed(), Status::Unknown]),
            failed()
        );
        assert_eq!(
            combine_peer_results([Status::Unknown, Status::Unknown]),
            Status::Unknown
        );
        assert_eq!(combine_peer_results([]), Status::Unknown);
    }
}
