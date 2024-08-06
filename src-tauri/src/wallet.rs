use chia::bls::PublicKey;
use sage::Database;

use crate::error::Result;

#[derive(Debug)]
pub struct Wallet {
    fingerprint: u32,
    intermediate_pk: PublicKey,
    db: Database,
}

impl Wallet {
    pub fn new(fingerprint: u32, intermediate_pk: PublicKey, db: Database) -> Self {
        Self {
            fingerprint,
            intermediate_pk,
            db,
        }
    }

    pub fn fingerprint(&self) -> u32 {
        self.fingerprint
    }

    pub async fn initial_sync(&self, derivation_batch_size: u32) -> Result<()> {
        let mut tx = self.db.tx().await?;

        let derivation_index = self.db.derivation_index(false).await?;

        if derivation_index < derivation_batch_size {
            tx.generate_unhardened_derivations(&self.intermediate_pk, derivation_batch_size)
                .await?;
        }

        tx.commit().await?;

        Ok(())
    }
}
