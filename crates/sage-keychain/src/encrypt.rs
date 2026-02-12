use aes_gcm::{AeadCore, Aes256Gcm, Key, KeyInit, Nonce, aead::Aead};
use argon2::Argon2;
use rand::{CryptoRng, Rng};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_with::{Bytes, serde_as};

use crate::KeychainError;

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Encrypted {
    #[serde_as(as = "Bytes")]
    pub ciphertext: Vec<u8>,
    #[serde_as(as = "Bytes")]
    pub nonce: Vec<u8>,
    #[serde_as(as = "Bytes")]
    pub salt: [u8; 32],
}

fn encryption_key(password: &[u8], salt: &[u8]) -> Result<Key<Aes256Gcm>, KeychainError> {
    let mut key_material = [0; 32];
    Argon2::default()
        .hash_password_into(password, salt, &mut key_material)
        .map_err(KeychainError::Argon2)?;
    Ok(*Key::<Aes256Gcm>::from_slice(&key_material))
}

pub fn encrypt(
    password: &[u8],
    rng: &mut (impl CryptoRng + Rng),
    data: &impl Serialize,
) -> Result<Encrypted, KeychainError> {
    let salt: [u8; 32] = rng.r#gen();
    let key = encryption_key(password, &salt)?;
    let cipher = Aes256Gcm::new(&key);
    let nonce = Aes256Gcm::generate_nonce(rng);

    let data = bincode::serialize(data)?;
    let ciphertext = cipher
        .encrypt(&nonce, data.as_ref())
        .map_err(|_| KeychainError::Encrypt)?;

    Ok(Encrypted {
        ciphertext,
        nonce: nonce.to_vec(),
        salt,
    })
}

pub fn decrypt<T>(encrypted: &Encrypted, password: &[u8]) -> Result<T, KeychainError>
where
    T: DeserializeOwned,
{
    let key = encryption_key(password, &encrypted.salt)?;
    let cipher = Aes256Gcm::new(&key);

    let nonce = Nonce::from_slice(&encrypted.nonce);
    let ciphertext = encrypted.ciphertext.as_ref();
    let data = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| KeychainError::Decrypt)?;

    Ok(bincode::deserialize(&data)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha20Rng;

    fn seeded_rng(seed: u64) -> ChaCha20Rng {
        ChaCha20Rng::seed_from_u64(seed)
    }

    #[test]
    fn encrypt_decrypt_round_trip() {
        let password = b"test-password";
        let mut rng = seeded_rng(42);
        let data: Vec<u8> = vec![1, 2, 3, 4, 5];

        let encrypted = encrypt(password, &mut rng, &data).unwrap();
        let decrypted: Vec<u8> = decrypt(&encrypted, password).unwrap();

        assert_eq!(data, decrypted);
    }

    #[test]
    fn wrong_password_fails() {
        let mut rng = seeded_rng(42);
        let data: Vec<u8> = vec![10, 20, 30];

        let encrypted = encrypt(b"correct", &mut rng, &data).unwrap();
        let result = decrypt::<Vec<u8>>(&encrypted, b"wrong");

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), KeychainError::Decrypt));
    }

    #[test]
    fn different_seeds_produce_different_salts() {
        let data: Vec<u8> = vec![1, 2, 3];

        let mut rng1 = seeded_rng(1);
        let mut rng2 = seeded_rng(2);

        let enc1 = encrypt(b"pass", &mut rng1, &data).unwrap();
        let enc2 = encrypt(b"pass", &mut rng2, &data).unwrap();

        assert_ne!(enc1.salt, enc2.salt);
    }

    #[test]
    fn empty_data_round_trip() {
        let mut rng = seeded_rng(99);
        let data: Vec<u8> = vec![];

        let encrypted = encrypt(b"pass", &mut rng, &data).unwrap();
        let decrypted: Vec<u8> = decrypt(&encrypted, b"pass").unwrap();

        assert_eq!(data, decrypted);
    }

    #[test]
    fn tampered_ciphertext_fails() {
        let mut rng = seeded_rng(42);
        let data: Vec<u8> = vec![1, 2, 3];

        let mut encrypted = encrypt(b"pass", &mut rng, &data).unwrap();

        // Tamper with ciphertext
        if let Some(byte) = encrypted.ciphertext.first_mut() {
            *byte ^= 0xFF;
        }

        let result = decrypt::<Vec<u8>>(&encrypted, b"pass");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), KeychainError::Decrypt));
    }

    #[test]
    fn tampered_nonce_fails() {
        let mut rng = seeded_rng(42);
        let data: Vec<u8> = vec![1, 2, 3];

        let mut encrypted = encrypt(b"pass", &mut rng, &data).unwrap();

        // Tamper with nonce
        if let Some(byte) = encrypted.nonce.first_mut() {
            *byte ^= 0xFF;
        }

        let result = decrypt::<Vec<u8>>(&encrypted, b"pass");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), KeychainError::Decrypt));
    }
}
