use chia::protocol::{CoinStateUpdate, Handshake, NewPeakWallet};

#[derive(Debug)]
pub enum Event {
    Handshake(Handshake),
    NewPeakWallet(NewPeakWallet),
    CoinStateUpdate(CoinStateUpdate),
}
