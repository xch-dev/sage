use std::time::Duration;

use chia::protocol::{Bytes32, SpendBundle};
use tokio::time::timeout;
use tracing::{info, warn};

use crate::{WalletError, WalletPeer};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    Pending,
    Success,
    Failed(u8, Option<String>),
    Unknown,
}

pub async fn submit_to_peers(
    peers: &[WalletPeer],
    genesis_challenge: Bytes32,
    spend_bundle: SpendBundle,
) -> Result<Status, WalletError> {
    let transaction_id = spend_bundle.name();

    info!(
        "Broadcasting transaction id {}: {:?}",
        transaction_id, spend_bundle
    );

    let mut mempool = false;
    let mut status = 0;
    let mut error = None;

    for peer in peers {
        let ip = peer.socket_addr().ip();
        match submit_transaction(peer, spend_bundle.clone(), genesis_challenge).await? {
            Status::Pending => {
                mempool = true;
            }
            Status::Success => {
                return Ok(Status::Success);
            }
            Status::Failed(failure_status, failure_error) => {
                info!("Transaction {transaction_id} failed according to peer {ip}, but will check other peers");
                status = failure_status;
                error = failure_error;
            }
            Status::Unknown => {}
        }
    }

    Ok(if mempool {
        Status::Pending
    } else {
        Status::Failed(status, error)
    })
}

pub async fn submit_transaction(
    peer: &WalletPeer,
    spend_bundle: SpendBundle,
    genesis_challenge: Bytes32,
) -> Result<Status, WalletError> {
    let ack = match timeout(
        Duration::from_secs(3),
        peer.send_transaction(spend_bundle.clone()),
    )
    .await
    {
        Ok(Ok(ack)) => ack,
        Err(_timeout) => {
            warn!("Send transaction timed out for {}", peer.socket_addr());
            return Ok(Status::Unknown);
        }
        Ok(Err(err)) => {
            warn!(
                "Send transaction failed for {}: {}",
                peer.socket_addr(),
                err
            );
            return Ok(Status::Unknown);
        }
    };

    info!(
        "Transaction sent to {} with ack {:?}",
        peer.socket_addr(),
        ack
    );

    if ack.status == 1 {
        return Ok(Status::Pending);
    };

    let coin_ids: Vec<Bytes32> = spend_bundle
        .coin_spends
        .iter()
        .map(|cs| cs.coin.coin_id())
        .collect();

    let coin_states = match timeout(
        Duration::from_secs(2),
        peer.fetch_coins(coin_ids.clone(), genesis_challenge),
    )
    .await
    {
        Ok(Ok(coin_states)) => coin_states,
        Err(_timeout) => {
            warn!("Coin lookup timed out for {}", peer.socket_addr());
            return Ok(Status::Unknown);
        }
        Ok(Err(err)) => {
            warn!("Coin lookup failed for {}: {}", peer.socket_addr(), err);
            return Ok(Status::Unknown);
        }
    };

    if coin_states.iter().all(|cs| cs.spent_height.is_some())
        && coin_ids
            .into_iter()
            .all(|coin_id| coin_states.iter().any(|cs| cs.coin.coin_id() == coin_id))
    {
        return Ok(Status::Success);
    } else if coin_states.iter().any(|cs| cs.spent_height.is_some()) {
        return Ok(Status::Failed(3, None));
    }

    Ok(Status::Failed(3, None))
}
