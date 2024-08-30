use chia::{bls::PublicKey, protocol::Bytes32};
use sage_database::Database;

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

    pub fn fingerprint(&self) -> u32 {
        self.fingerprint
    }
}
