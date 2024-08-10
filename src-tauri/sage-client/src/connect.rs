use std::net::SocketAddr;

use chia::{
    protocol::{Handshake, Message, NodeType, ProtocolMessageTypes},
    traits::Streamable,
};
use native_tls::TlsConnector;
use tokio::sync::mpsc;

use crate::{Error, Peer, Result};

pub async fn connect_peer(
    network_id: String,
    tls_connector: TlsConnector,
    socket_addr: SocketAddr,
) -> Result<(Peer, mpsc::Receiver<Message>)> {
    let (peer, mut receiver) = Peer::connect(socket_addr, tls_connector).await?;

    peer.send(Handshake {
        network_id: network_id.clone(),
        protocol_version: "0.0.37".to_string(),
        software_version: "0.0.0".to_string(),
        server_port: 0,
        node_type: NodeType::Wallet,
        capabilities: vec![
            (1, "1".to_string()),
            (2, "1".to_string()),
            (3, "1".to_string()),
        ],
    })
    .await?;

    let Some(message) = receiver.recv().await else {
        return Err(Error::MissingHandshake);
    };

    if message.msg_type != ProtocolMessageTypes::Handshake {
        return Err(Error::InvalidResponse(
            vec![ProtocolMessageTypes::Handshake],
            message.msg_type,
        ));
    }

    let handshake = Handshake::from_bytes(&message.data)?;

    if handshake.node_type != NodeType::FullNode {
        return Err(Error::WrongNodeType(
            NodeType::FullNode,
            handshake.node_type,
        ));
    }

    if handshake.network_id != network_id {
        return Err(Error::WrongNetwork(network_id, handshake.network_id));
    }

    Ok((peer, receiver))
}
