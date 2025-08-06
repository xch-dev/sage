use chia::protocol::SpendBundle;
use tracing::{info, warn};

use crate::{WalletError, WalletPeer};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    Pending,
    Failed(u8, Option<String>),
    Unknown,
}

pub async fn submit_to_peers(
    peers: &[WalletPeer],
    spend_bundle: SpendBundle,
) -> Result<Status, WalletError> {
    let transaction_id = spend_bundle.name();

    info!("Checking transaction id {}", transaction_id);

    let mut mempool = false;
    let mut failed = false;
    let mut status = 0;
    let mut error = None;

    for peer in peers {
        let ip = peer.socket_addr().ip();
        match submit_transaction(peer, spend_bundle.clone()).await? {
            Status::Pending => {
                mempool = true;
            }
            Status::Failed(failure_status, failure_error) => {
                info!("Transaction {transaction_id} failed according to peer {ip}, but will check other peers");
                status = failure_status;
                error = failure_error;
                failed = true;
            }
            Status::Unknown => {}
        }
    }

    Ok(if mempool {
        Status::Pending
    } else if failed {
        Status::Failed(status, error)
    } else {
        Status::Unknown
    })
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

    if ack.status == 1 {
        return Ok(Status::Pending);
    };

    Ok(Status::Failed(ack.status, ack.error))
}
