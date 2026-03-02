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

    const TEST_MNEMONIC: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    const PASSWORD: &[u8] = b"test-password";

    fn test_mnemonic() -> Mnemonic {
        Mnemonic::parse(TEST_MNEMONIC).unwrap()
    }

    #[test]
    fn add_mnemonic_and_extract_round_trip() {
        let mut keychain = Keychain::default();
        let mnemonic = test_mnemonic();

        let fingerprint = keychain.add_mnemonic(&mnemonic, PASSWORD).unwrap();
        assert!(keychain.contains(fingerprint));
        assert!(keychain.has_secret_key(fingerprint));

        let (extracted_mnemonic, extracted_sk) =
            keychain.extract_secrets(fingerprint, PASSWORD).unwrap();

        assert_eq!(extracted_mnemonic.unwrap().to_string(), mnemonic.to_string());
        assert!(extracted_sk.is_some());
    }

    #[test]
    fn add_secret_key_extract_has_no_mnemonic() {
        let mut keychain = Keychain::default();
        let mnemonic = test_mnemonic();
        let sk = SecretKey::from_seed(&mnemonic.to_seed(""));

        let fingerprint = keychain.add_secret_key(&sk, PASSWORD).unwrap();
        assert!(keychain.has_secret_key(fingerprint));

        let (extracted_mnemonic, extracted_sk) =
            keychain.extract_secrets(fingerprint, PASSWORD).unwrap();

        // Mnemonic should be None when added as raw secret key
        assert!(extracted_mnemonic.is_none());
        assert!(extracted_sk.is_some());
    }

    #[test]
    fn add_public_key_has_no_secret() {
        let mut keychain = Keychain::default();
        let mnemonic = test_mnemonic();
        let sk = SecretKey::from_seed(&mnemonic.to_seed(""));
        let pk = sk.public_key();

        let fingerprint = keychain.add_public_key(&pk).unwrap();
        assert!(keychain.contains(fingerprint));
        assert!(!keychain.has_secret_key(fingerprint));

        let (mnemonic_out, sk_out) =
            keychain.extract_secrets(fingerprint, PASSWORD).unwrap();
        assert!(mnemonic_out.is_none());
        assert!(sk_out.is_none());
    }

    #[test]
    fn duplicate_key_returns_error() {
        let mut keychain = Keychain::default();
        let mnemonic = test_mnemonic();

        keychain.add_mnemonic(&mnemonic, PASSWORD).unwrap();
        let result = keychain.add_mnemonic(&mnemonic, PASSWORD);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), KeychainError::KeyExists));
    }

    #[test]
    fn remove_key() {
        let mut keychain = Keychain::default();
        let mnemonic = test_mnemonic();
        let fingerprint = keychain.add_mnemonic(&mnemonic, PASSWORD).unwrap();

        assert!(keychain.contains(fingerprint));
        assert!(keychain.remove(fingerprint));
        assert!(!keychain.contains(fingerprint));
        assert!(!keychain.remove(fingerprint)); // Already removed
    }

    #[test]
    fn wrong_password_fails_extract() {
        let mut keychain = Keychain::default();
        let mnemonic = test_mnemonic();
        let fingerprint = keychain.add_mnemonic(&mnemonic, PASSWORD).unwrap();

        let result = keychain.extract_secrets(fingerprint, b"wrong-password");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), KeychainError::Decrypt));
    }

    #[test]
    fn fingerprints_iteration() {
        let mut keychain = Keychain::default();

        // Add 3 different keys using different mnemonics
        let m1 = Mnemonic::parse("abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about").unwrap();
        let m2 = Mnemonic::parse("zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo wrong").unwrap();
        let sk3 = SecretKey::from_seed(&[42u8; 64]);
        let pk3 = sk3.public_key();

        let f1 = keychain.add_mnemonic(&m1, PASSWORD).unwrap();
        let f2 = keychain.add_mnemonic(&m2, PASSWORD).unwrap();
        let f3 = keychain.add_public_key(&pk3).unwrap();

        let mut fps: Vec<u32> = keychain.fingerprints().collect();
        fps.sort();

        let mut expected = vec![f1, f2, f3];
        expected.sort();

        assert_eq!(fps, expected);
        assert_eq!(fps.len(), 3);
    }

    #[test]
    fn extract_public_key() {
        let mut keychain = Keychain::default();
        let mnemonic = test_mnemonic();
        let sk = SecretKey::from_seed(&mnemonic.to_seed(""));
        let pk = sk.public_key();

        let fingerprint = keychain.add_mnemonic(&mnemonic, PASSWORD).unwrap();
        let extracted_pk = keychain.extract_public_key(fingerprint).unwrap();

        assert_eq!(extracted_pk.unwrap(), pk);
    }

    #[test]
    fn serialization_round_trip() {
        let mut keychain = Keychain::default();
        let mnemonic = test_mnemonic();
        let fingerprint = keychain.add_mnemonic(&mnemonic, PASSWORD).unwrap();

        let bytes = keychain.to_bytes().unwrap();
        let restored = Keychain::from_bytes(&bytes).unwrap();

        assert!(restored.contains(fingerprint));
        assert!(restored.has_secret_key(fingerprint));

        let (extracted_mnemonic, _) =
            restored.extract_secrets(fingerprint, PASSWORD).unwrap();
        assert_eq!(
            extracted_mnemonic.unwrap().to_string(),
            mnemonic.to_string()
        );
    }

    #[test]
    fn extract_nonexistent_key() {
        let keychain = Keychain::default();
        let (mnemonic, sk) = keychain.extract_secrets(999999, PASSWORD).unwrap();
        assert!(mnemonic.is_none());
        assert!(sk.is_none());
    }
}
