use std::collections::HashMap;

use bip39::Mnemonic;
use chia_wallet_sdk::prelude::*;
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use serde::{Deserialize, Serialize};
use serde_with::{Bytes, serde_as};

use crate::{
    KeychainError,
    encrypt::{Encrypted, decrypt, encrypt},
    key_data::{KeyData, SecretKeyData},
};

/// Legacy KeyData without password_protected field, for backward compat
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[repr(u8)]
enum LegacyKeyData {
    Public {
        #[serde_as(as = "Bytes")]
        master_pk: [u8; 48],
    },
    Secret {
        #[serde_as(as = "Bytes")]
        master_pk: [u8; 48],
        entropy: bool,
        encrypted: Encrypted,
    },
}

impl From<LegacyKeyData> for KeyData {
    fn from(legacy: LegacyKeyData) -> Self {
        match legacy {
            LegacyKeyData::Public { master_pk } => KeyData::Public { master_pk },
            LegacyKeyData::Secret {
                master_pk,
                entropy,
                encrypted,
            } => KeyData::Secret {
                master_pk,
                entropy,
                encrypted,
                password_protected: false,
            },
        }
    }
}

#[derive(Debug)]
pub struct Keychain {
    rng: ChaCha20Rng,
    keys: HashMap<u32, KeyData>,
}

impl Default for Keychain {
    fn default() -> Self {
        Self {
            rng: ChaCha20Rng::from_entropy(),
            keys: HashMap::default(),
        }
    }
}

impl Keychain {
    pub fn from_bytes(data: &[u8]) -> Result<Self, KeychainError> {
        let keys: HashMap<u32, KeyData> = bincode::deserialize(data).or_else(|_| {
            let legacy: HashMap<u32, LegacyKeyData> = bincode::deserialize(data)?;
            Ok::<_, bincode::Error>(legacy.into_iter().map(|(k, v)| (k, v.into())).collect())
        })?;
        Ok(Self {
            rng: ChaCha20Rng::from_entropy(),
            keys,
        })
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, KeychainError> {
        Ok(bincode::serialize(&self.keys)?)
    }

    pub fn contains(&self, fingerprint: u32) -> bool {
        self.keys.contains_key(&fingerprint)
    }

    pub fn remove(&mut self, fingerprint: u32) -> bool {
        self.keys.remove(&fingerprint).is_some()
    }

    pub fn fingerprints(&self) -> impl Iterator<Item = u32> + '_ {
        self.keys.keys().copied()
    }

    pub fn extract_public_key(
        &self,
        fingerprint: u32,
    ) -> Result<Option<PublicKey>, KeychainError> {
        match self.keys.get(&fingerprint) {
            Some(KeyData::Public { master_pk } | KeyData::Secret { master_pk, .. }) => {
                Ok(Some(PublicKey::from_bytes(master_pk)?))
            }
            None => Ok(None),
        }
    }

    pub fn extract_secrets(
        &self,
        fingerprint: u32,
        password: &[u8],
    ) -> Result<(Option<Mnemonic>, Option<SecretKey>), KeychainError> {
        match self.keys.get(&fingerprint) {
            Some(KeyData::Public { .. }) | None => Ok((None, None)),
            Some(KeyData::Secret {
                entropy, encrypted, ..
            }) => {
                let data = decrypt::<SecretKeyData>(encrypted, password)?;

                let mnemonic = if *entropy {
                    Some(Mnemonic::from_entropy(&data.0)?)
                } else {
                    None
                };

                let secret_key = if let Some(mnemonic) = mnemonic.as_ref() {
                    SecretKey::from_seed(&mnemonic.to_seed(""))
                } else {
                    SecretKey::from_bytes(&data.0.try_into().expect("invalid length"))?
                };

                Ok((mnemonic, Some(secret_key)))
            }
        }
    }

    pub fn has_secret_key(&self, fingerprint: u32) -> bool {
        let Some(key_data) = self.keys.get(&fingerprint) else {
            return false;
        };

        match key_data {
            KeyData::Public { .. } => false,
            KeyData::Secret { .. } => true,
        }
    }

    pub fn is_password_protected(&self, fingerprint: u32) -> bool {
        matches!(
            self.keys.get(&fingerprint),
            Some(KeyData::Secret {
                password_protected: true,
                ..
            })
        )
    }

    pub fn add_public_key(&mut self, master_pk: &PublicKey) -> Result<u32, KeychainError> {
        let fingerprint = master_pk.get_fingerprint();

        if self.contains(fingerprint) {
            return Err(KeychainError::KeyExists);
        }

        self.keys.insert(
            fingerprint,
            KeyData::Public {
                master_pk: master_pk.to_bytes(),
            },
        );

        Ok(fingerprint)
    }

    pub fn add_secret_key(
        &mut self,
        master_sk: &SecretKey,
        password: &[u8],
    ) -> Result<u32, KeychainError> {
        let master_pk = master_sk.public_key();
        let fingerprint = master_pk.get_fingerprint();

        if self.contains(fingerprint) {
            return Err(KeychainError::KeyExists);
        }

        let encrypted = encrypt(
            password,
            &mut self.rng,
            &SecretKeyData(master_sk.to_bytes().to_vec()),
        )?;

        self.keys.insert(
            fingerprint,
            KeyData::Secret {
                master_pk: master_pk.to_bytes(),
                entropy: false,
                encrypted,
                password_protected: !password.is_empty(),
            },
        );

        Ok(fingerprint)
    }

    pub fn add_mnemonic(
        &mut self,
        mnemonic: &Mnemonic,
        password: &[u8],
    ) -> Result<u32, KeychainError> {
        let entropy = mnemonic.to_entropy();
        let seed = mnemonic.to_seed("");
        let master_sk = SecretKey::from_seed(&seed);
        let master_pk = master_sk.public_key();
        let fingerprint = master_pk.get_fingerprint();

        if self.contains(fingerprint) {
            return Err(KeychainError::KeyExists);
        }

        let encrypted = encrypt(password, &mut self.rng, &SecretKeyData(entropy))?;

        self.keys.insert(
            fingerprint,
            KeyData::Secret {
                master_pk: master_pk.to_bytes(),
                entropy: true,
                encrypted,
                password_protected: !password.is_empty(),
            },
        );

        Ok(fingerprint)
    }

}
