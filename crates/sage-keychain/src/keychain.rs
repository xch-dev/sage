use std::collections::HashMap;

use bip39::Mnemonic;
use chia_wallet_sdk::prelude::*;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;

use crate::{
    KeychainError,
    encrypt::{decrypt, encrypt},
    key_data::{KeyData, SecretKeyData},
};

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
        let keys = bincode::deserialize(data)?;
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

    pub fn extract_public_key(&self, fingerprint: u32) -> Result<Option<PublicKey>, KeychainError> {
        match self.keys.get(&fingerprint) {
            Some(KeyData::Public { master_pk } | KeyData::Secret { master_pk, .. }) => {
                Ok(Some(PublicKey::from_bytes(master_pk)?))
            }
            Some(KeyData::Vault { .. } | KeyData::Watch { .. }) | None => Ok(None),
        }
    }

    pub fn extract_secrets(
        &self,
        fingerprint: u32,
        password: &[u8],
    ) -> Result<(Option<Mnemonic>, Option<SecretKey>), KeychainError> {
        match self.keys.get(&fingerprint) {
            Some(KeyData::Public { .. } | KeyData::Vault { .. } | KeyData::Watch { .. }) | None => {
                Ok((None, None))
            }
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

    pub fn extract_vault_id(&self, fingerprint: u32) -> Option<Bytes32> {
        match self.keys.get(&fingerprint) {
            Some(KeyData::Vault { launcher_id }) => Some(Bytes32::new(*launcher_id)),
            Some(KeyData::Public { .. } | KeyData::Secret { .. } | KeyData::Watch { .. })
            | None => None,
        }
    }

    pub fn extract_watch_p2_puzzle_hashes(&self, fingerprint: u32) -> Option<Vec<Bytes32>> {
        match self.keys.get(&fingerprint) {
            Some(KeyData::Watch { p2_puzzle_hashes }) => Some(
                p2_puzzle_hashes
                    .iter()
                    .map(|p2_puzzle_hash| Bytes32::new(*p2_puzzle_hash))
                    .collect(),
            ),
            Some(KeyData::Public { .. } | KeyData::Secret { .. } | KeyData::Vault { .. })
            | None => None,
        }
    }

    pub fn has_secret_key(&self, fingerprint: u32) -> bool {
        let Some(key_data) = self.keys.get(&fingerprint) else {
            return false;
        };

        match key_data {
            KeyData::Public { .. } | KeyData::Vault { .. } | KeyData::Watch { .. } => false,
            KeyData::Secret { .. } => true,
        }
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
            },
        );

        Ok(fingerprint)
    }

    pub fn add_vault(&mut self, launcher_id: &Bytes32) -> Result<u32, KeychainError> {
        let fingerprint = u32::from_be_bytes([
            launcher_id[0],
            launcher_id[1],
            launcher_id[2],
            launcher_id[3],
        ]);

        if self.contains(fingerprint) {
            return Err(KeychainError::KeyExists);
        }

        self.keys.insert(
            fingerprint,
            KeyData::Vault {
                launcher_id: launcher_id.to_bytes(),
            },
        );

        Ok(fingerprint)
    }

    pub fn add_watch_p2_puzzle_hashes(
        &mut self,
        p2_puzzle_hashes: &[Bytes32],
    ) -> Result<u32, KeychainError> {
        let fingerprint = u32::from_be_bytes(self.rng.r#gen::<[u8; 4]>());

        if self.contains(fingerprint) {
            return Err(KeychainError::KeyExists);
        }

        let p2_puzzle_hashes = p2_puzzle_hashes
            .iter()
            .map(|p2_puzzle_hash| p2_puzzle_hash.to_bytes())
            .collect();

        self.keys
            .insert(fingerprint, KeyData::Watch { p2_puzzle_hashes });

        Ok(fingerprint)
    }
}
