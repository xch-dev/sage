use std::{net::IpAddr, sync::Arc};

use chia::protocol::Message;
use chia_wallet_sdk::{Network, NetworkId};

use crate::Wallet;

#[derive(Debug)]
pub enum SyncCommand {
    SwitchWallet {
        wallet: Option<Arc<Wallet>>,
    },
    SwitchNetwork {
        network_id: NetworkId,
        network: Network,
    },
    HandleMessage {
        ip: IpAddr,
        message: Message,
    },
    ConnectionClosed(IpAddr),
    SetDiscoverPeers(bool),
    SetTargetPeers(usize),
}
