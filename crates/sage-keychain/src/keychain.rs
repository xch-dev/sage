use std::collections::HashMap;

use crate::{
    KeychainError,
    encrypt::{decrypt, encrypt},
    key_data::{KeyData, SecretKeyData},
};
use bip39::Mnemonic;
use chia_wallet_sdk::prelude::*;
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;

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
        let keys: HashMap<u32, KeyData> = bincode::deserialize(data)?;
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

    /// Probes whether the key is password-protected by attempting to
    /// decrypt with an empty password. Returns `false` for public keys.
    pub fn is_password_protected(&self, fingerprint: u32) -> bool {
        match self.keys.get(&fingerprint) {
            Some(KeyData::Secret { encrypted, .. }) => {
                decrypt::<SecretKeyData>(encrypted, b"").is_err()
            }
            _ => false,
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

    pub fn change_password(
        &mut self,
        fingerprint: u32,
        old_password: &[u8],
        new_password: &[u8],
    ) -> Result<(), KeychainError> {
        let key_data = self
            .keys
            .get(&fingerprint)
            .ok_or(KeychainError::KeyNotFound)?;

        let (entropy, master_pk, secret_data) = match key_data {
            KeyData::Public { .. } => return Err(KeychainError::NoSecretKey),
            KeyData::Secret {
                entropy,
                master_pk,
                encrypted,
                ..
            } => {
                let data = decrypt::<SecretKeyData>(encrypted, old_password)?;
                (*entropy, *master_pk, data)
            }
        };

        let encrypted = encrypt(new_password, &mut self.rng, &secret_data)?;

        self.keys.insert(
            fingerprint,
            KeyData::Secret {
                master_pk,
                entropy,
                encrypted,
            },
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bip39::Mnemonic;

    #[test]
    fn test_change_password() {
        let mut keychain = Keychain::default();
        let mnemonic = Mnemonic::from_entropy(&[0u8; 16]).unwrap();
        let fingerprint = keychain.add_mnemonic(&mnemonic, b"").unwrap();

        keychain
            .change_password(fingerprint, b"", b"secret123")
            .unwrap();
        assert!(keychain.extract_secrets(fingerprint, b"").is_err());

        let (mnemonic_out, Some(_sk)) =
            keychain.extract_secrets(fingerprint, b"secret123").unwrap()
        else {
            panic!("expected secret key");
        };
        assert!(mnemonic_out.is_some());

        keychain
            .change_password(fingerprint, b"secret123", b"newpass")
            .unwrap();
        assert!(keychain.extract_secrets(fingerprint, b"secret123").is_err());
        let (_m, Some(_sk)) = keychain.extract_secrets(fingerprint, b"newpass").unwrap() else {
            panic!("expected secret key");
        };

        keychain
            .change_password(fingerprint, b"newpass", b"")
            .unwrap();
        let (_m, Some(_sk)) = keychain.extract_secrets(fingerprint, b"").unwrap() else {
            panic!("expected secret key");
        };
    }

    #[test]
    fn test_change_password_wrong_old_password() {
        let mut keychain = Keychain::default();
        let mnemonic = Mnemonic::from_entropy(&[0u8; 16]).unwrap();
        let fingerprint = keychain.add_mnemonic(&mnemonic, b"correct").unwrap();
        assert!(
            keychain
                .change_password(fingerprint, b"wrong", b"newpass")
                .is_err()
        );
        let (_m, Some(_sk)) = keychain.extract_secrets(fingerprint, b"correct").unwrap() else {
            panic!("expected secret key");
        };
    }

    #[test]
    fn test_change_password_public_key_fails() {
        let mut keychain = Keychain::default();
        let mnemonic = Mnemonic::from_entropy(&[0u8; 16]).unwrap();
        let master_sk = SecretKey::from_seed(&mnemonic.to_seed(""));
        let master_pk = master_sk.public_key();
        let fingerprint = keychain.add_public_key(&master_pk).unwrap();
        assert!(keychain.change_password(fingerprint, b"", b"pass").is_err());
    }

    #[test]
    fn test_serialization_roundtrip_with_password() {
        let mut keychain = Keychain::default();
        let mnemonic = Mnemonic::from_entropy(&[0u8; 16]).unwrap();
        let fingerprint = keychain.add_mnemonic(&mnemonic, b"pass123").unwrap();
        let bytes = keychain.to_bytes().unwrap();
        let keychain2 = Keychain::from_bytes(&bytes).unwrap();
        let (_m, Some(_sk)) = keychain2.extract_secrets(fingerprint, b"pass123").unwrap() else {
            panic!("expected secret key");
        };
    }
}
