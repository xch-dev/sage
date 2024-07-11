use chia::bls::PublicKey;

use crate::Database;

#[derive(Debug)]
pub struct Wallet {
    db: Database,
    intermediate_pk: PublicKey,
}

impl Wallet {
    pub fn new(db: Database, intermediate_pk: PublicKey) -> Self {
        Self {
            db,
            intermediate_pk,
        }
    }
}
