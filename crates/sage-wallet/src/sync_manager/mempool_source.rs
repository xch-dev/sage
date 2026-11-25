use std::{sync::Arc, time::Duration};

use chia_wallet_sdk::{
    coinset::{ChiaRpcClient, CoinsetClient},
    prelude::*,
};
use futures_util::future::BoxFuture;
use sage_config::{InheritedNetwork, Network};
use tokio::time::timeout;
use tracing::debug;

use crate::WalletError;

/// How long a request to the Coinset API can take.
const COINSET_TIMEOUT: Duration = Duration::from_secs(10);

/// Where the spend bundles of announced mempool transactions are fetched from. CHIP-26 only
/// announces transaction ids, and full nodes only answer `RequestTransaction` from other full
/// nodes, banning a wallet that sends it, so the spend bundles come from somewhere else.
pub trait MempoolSource: Send + Sync {
    /// Returns the spend bundle of the transaction, or `None` if it isn't in the mempool. A
    /// source can lag behind the node that announced the transaction, so `None` may only mean
    /// it isn't there yet.
    fn spend_bundle(
        &self,
        transaction_id: Bytes32,
    ) -> BoxFuture<'_, Result<Option<SpendBundle>, WalletError>>;
}

/// A client for the network's [Coinset](https://coinset.org) API, or `None` for a network that
/// Coinset doesn't serve.
pub fn coinset_client(network: &Network) -> Option<CoinsetClient> {
    Some(match network.same_chain_as()? {
        InheritedNetwork::Mainnet => CoinsetClient::mainnet(),
        InheritedNetwork::Testnet11 => CoinsetClient::testnet11(),
    })
}

/// Looks up mempool transactions with the network's Coinset API, or returns `None` for a
/// network that Coinset doesn't serve.
pub fn coinset_mempool(network: &Network) -> Option<Arc<dyn MempoolSource>> {
    let client = coinset_client(network)?;
    Some(Arc::new(CoinsetMempool { client }))
}

struct CoinsetMempool {
    client: CoinsetClient,
}

impl CoinsetMempool {
    async fn fetch(&self, transaction_id: Bytes32) -> Result<Option<SpendBundle>, WalletError> {
        let response = timeout(
            COINSET_TIMEOUT,
            self.client.get_mempool_item_by_tx_id(transaction_id),
        )
        .await?
        .map_err(|error| WalletError::Coinset(error.to_string()))?;

        if response.mempool_item.is_none()
            && let Some(error) = response.error
        {
            debug!("Coinset doesn't have mempool item {transaction_id}: {error}");
        }

        Ok(response.mempool_item.map(|item| item.spend_bundle))
    }
}

impl MempoolSource for CoinsetMempool {
    fn spend_bundle(
        &self,
        transaction_id: Bytes32,
    ) -> BoxFuture<'_, Result<Option<SpendBundle>, WalletError>> {
        Box::pin(self.fetch(transaction_id))
    }
}
