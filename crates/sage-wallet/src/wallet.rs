use chia::{bls::PublicKey, protocol::Bytes32};
use sage_database::Database;

mod cat_coin_management;
mod cat_spends;
mod cats;
mod coin_selection;
mod derivations;
mod dids;
mod make_offer;
mod nfts;
mod offer;
mod p2_coin_management;
mod p2_send;
mod p2_spends;
mod signing;

pub use nfts::WalletNftMint;
pub use offer::*;

#[derive(Debug)]
pub struct Wallet {
    pub db: Database,
    pub fingerprint: u32,
    pub intermediate_pk: PublicKey,
    pub genesis_challenge: Bytes32,
}

impl Wallet {
    pub fn new(
        db: Database,
        fingerprint: u32,
        intermediate_pk: PublicKey,
        genesis_challenge: Bytes32,
    ) -> Self {
        Self {
            db,
            fingerprint,
            intermediate_pk,
            genesis_challenge,
        }
    }
}
