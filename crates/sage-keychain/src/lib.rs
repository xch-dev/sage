use aes_gcm::{aead::Aead, AeadCore, Aes256Gcm, Key, KeyInit, Nonce};
use argon2::Argon2;
use rand::{CryptoRng, Rng};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_with::{serde_as, Bytes};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum KeychainError {
    #[error("Encoding error: {0}")]
    Bincode(#[from] bincode::Error),

    #[error("Could not encrypt key data.")]
    Encrypt,

    #[error("Could not decrypt key data.")]
    Decrypt,

    #[error("Argon2 error: {0}")]
    Argon2(argon2::Error),
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[repr(u8)]
pub enum KeyData {
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

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretKeyData(#[serde_as(as = "Bytes")] pub Vec<u8>);

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
    let salt: [u8; 32] = rng.gen();
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
