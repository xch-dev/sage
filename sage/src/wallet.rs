use chia::bls::PublicKey;

use crate::Database;

#[derive(Debug, Clone, Copy)]
pub struct WalletOptions {
    min_derivations: u32,
    derivation_batch_size: u32,
}

impl Default for WalletOptions {
    fn default() -> Self {
        Self {
            min_derivations: 500,
            derivation_batch_size: 500,
        }
    }
}

#[derive(Debug)]
pub struct Wallet {
    db: Database,
    intermediate_pk: PublicKey,
    options: WalletOptions,
}

impl Wallet {
    pub fn new(db: Database, intermediate_pk: PublicKey, options: WalletOptions) -> Self {
        Self {
            db,
            intermediate_pk,
            options,
        }
    }
}
