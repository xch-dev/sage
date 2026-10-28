use std::collections::HashMap;

use bip39::Mnemonic;
use chia_wallet_sdk::prelude::*;
use rand::SeedableRng;
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
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A keychain written by an earlier version of Sage, holding a mnemonic, a secret key and a
    /// public key, all encrypted with an empty password.
    const KEYS_V0: &[u8] = include_bytes!("../fixtures/keys_v0.bin");

    #[test]
    fn loads_existing_keychain() -> anyhow::Result<()> {
        let keychain = Keychain::from_bytes(KEYS_V0)?;

        let mut fingerprints: Vec<u32> = keychain.fingerprints().collect();
        fingerprints.sort_unstable();
        assert_eq!(fingerprints, [350_302_556, 2_397_551_190, 2_916_568_590]);

        let mnemonic = Mnemonic::from_entropy(&[7; 32])?;
        let (Some(extracted), Some(secret_key)) = keychain.extract_secrets(2_916_568_590, b"")?
        else {
            panic!("mnemonic secrets are missing");
        };
        assert_eq!(extracted, mnemonic);
        assert_eq!(
            secret_key.public_key(),
            SecretKey::from_seed(&mnemonic.to_seed("")).public_key()
        );

        let secret_key = SecretKey::from_seed(&[8; 32]);
        let (None, Some(extracted)) = keychain.extract_secrets(2_397_551_190, b"")? else {
            panic!("secret key is missing");
        };
        assert_eq!(extracted, secret_key);

        let public_key = SecretKey::from_seed(&[9; 32]).public_key();
        assert!(!keychain.has_secret_key(350_302_556));
        assert_eq!(keychain.extract_public_key(350_302_556)?, Some(public_key));

        // Saving it again must not change how existing keys are encoded.
        let reloaded = Keychain::from_bytes(&keychain.to_bytes()?)?;
        assert_eq!(reloaded.to_bytes()?.len(), KEYS_V0.len());
        assert_eq!(reloaded.extract_public_key(350_302_556)?, Some(public_key));

        Ok(())
    }
}
