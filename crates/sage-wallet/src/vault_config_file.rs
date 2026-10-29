//! Chia Cloud Wallet's vault configuration file, as exported by `getVaultConfigDownload` and
//! validated by `vaultConfigSchema` in ent-wallet.
//!
//! ```json
//! {
//!   "launcherId": "0x…",
//!   "vaultPuzzleVersion": "FORCE_SINGLETON_RECREATION",
//!   "custody": { "threshold": 1, "members": [{ "type": "publicKey", "publicKey": "0x…", "curve": "SECP256R1" }] },
//!   "recovery": { "threshold": 1, "clawbackTimelock": 86400, "members": [{ "type": "vault", "launcherId": "0x…" }] }
//! }
//! ```
//!
//! The version is optional, in which case either version may match the vault. Members don't say
//! whether they fast forward, since that follows from their curve in Cloud Wallet's layout.

use chia_wallet_sdk::{driver::VaultVersion, prelude::*};
use sage_database::{VaultConfig, VaultMember, VaultSigners};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum VaultConfigFileError {
    #[error("The vault config file isn't valid: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Invalid hex value in the vault config file: {0}")]
    Hex(String),

    #[error("Invalid public key in the vault config file: {0}")]
    PublicKey(String),

    #[error("The vault config file has an invalid threshold or timelock")]
    InvalidSigners,

    #[error("Custom members can't be written to a vault config file")]
    CustomMember,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ConfigFile {
    launcher_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    vault_puzzle_version: Option<PuzzleVersion>,
    custody: CustodyFile,
    recovery: RecoveryFile,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CustodyFile {
    threshold: usize,
    members: Vec<MemberFile>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RecoveryFile {
    threshold: usize,
    clawback_timelock: u64,
    members: Vec<MemberFile>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum MemberFile {
    #[serde(rename_all = "camelCase")]
    PublicKey {
        public_key: String,
        curve: Curve,
        /// How Cloud Wallet's app holds the key. It doesn't affect the vault's puzzle.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        key_type: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    Vault { launcher_id: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum Curve {
    #[serde(rename = "BLS12_381")]
    Bls12381,
    Secp256k1,
    Secp256r1,
    /// A passkey.
    Webauthn,
}

/// The puzzle version, which older files write as a number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
enum PuzzleVersion {
    Name(PuzzleVersionName),
    Number(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum PuzzleVersionName {
    Legacy,
    ForceSingletonRecreation,
}

/// A vault config read from a file, which must still be checked against the vault on chain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedVaultConfigFile {
    pub launcher_id: Bytes32,
    /// The configs the file can describe. There are two when the file doesn't say which puzzle
    /// version the vault uses, of which only one can rebuild the vault's custody hash.
    pub candidates: Vec<VaultConfig>,
}

/// Parses a vault config file, as exported by Chia Cloud Wallet.
pub fn parse_vault_config_file(json: &str) -> Result<ParsedVaultConfigFile, VaultConfigFileError> {
    let file: ConfigFile = serde_json::from_str(json)?;

    let custody = parse_signers(file.custody.threshold, file.custody.members)?;
    let recovery = parse_signers(file.recovery.threshold, file.recovery.members)?;

    if file.recovery.clawback_timelock == 0 {
        return Err(VaultConfigFileError::InvalidSigners);
    }

    let versions = match file.vault_puzzle_version {
        Some(PuzzleVersion::Name(PuzzleVersionName::Legacy) | PuzzleVersion::Number(1)) => {
            vec![VaultVersion::Legacy]
        }
        Some(
            PuzzleVersion::Name(PuzzleVersionName::ForceSingletonRecreation)
            | PuzzleVersion::Number(2),
        ) => vec![VaultVersion::ForceSingletonRecreation],
        Some(PuzzleVersion::Number(_)) => return Err(VaultConfigFileError::InvalidSigners),
        None => vec![VaultVersion::Legacy, VaultVersion::ForceSingletonRecreation],
    };

    Ok(ParsedVaultConfigFile {
        launcher_id: parse_bytes32(&file.launcher_id)?,
        candidates: versions
            .into_iter()
            .map(|version| VaultConfig {
                custody: custody.clone(),
                recovery: recovery.clone(),
                recovery_timelock: file.recovery.clawback_timelock,
                version,
            })
            .collect(),
    })
}

/// Writes a vault config file, in the format Chia Cloud Wallet exports and imports.
pub fn vault_config_file(
    launcher_id: Bytes32,
    config: &VaultConfig,
) -> Result<String, VaultConfigFileError> {
    let file = ConfigFile {
        launcher_id: hex_string(&launcher_id),
        vault_puzzle_version: Some(PuzzleVersion::Name(match config.version {
            VaultVersion::Legacy => PuzzleVersionName::Legacy,
            VaultVersion::ForceSingletonRecreation => PuzzleVersionName::ForceSingletonRecreation,
        })),
        custody: CustodyFile {
            threshold: config.custody.threshold,
            members: write_members(&config.custody.members)?,
        },
        recovery: RecoveryFile {
            threshold: config.recovery.threshold,
            clawback_timelock: config.recovery_timelock,
            members: write_members(&config.recovery.members)?,
        },
    };

    Ok(serde_json::to_string_pretty(&file)?)
}

fn parse_signers(
    threshold: usize,
    members: Vec<MemberFile>,
) -> Result<VaultSigners, VaultConfigFileError> {
    if members.is_empty() || threshold == 0 || threshold > members.len() {
        return Err(VaultConfigFileError::InvalidSigners);
    }

    let members = members
        .into_iter()
        .map(|member| {
            Ok(match member {
                MemberFile::PublicKey {
                    public_key, curve, ..
                } => {
                    let bytes = parse_hex(&public_key)?;
                    let invalid = || VaultConfigFileError::PublicKey(public_key.clone());

                    match curve {
                        Curve::Bls12381 => VaultMember::bls(
                            PublicKey::from_bytes(&bytes.try_into().map_err(|_| invalid())?)
                                .map_err(|_| invalid())?,
                        ),
                        Curve::Secp256k1 => VaultMember::secp256k1(
                            K1PublicKey::from_bytes(&bytes.try_into().map_err(|_| invalid())?)
                                .map_err(|_| invalid())?,
                        ),
                        Curve::Secp256r1 => VaultMember::secp256r1(
                            R1PublicKey::from_bytes(&bytes.try_into().map_err(|_| invalid())?)
                                .map_err(|_| invalid())?,
                        ),
                        Curve::Webauthn => VaultMember::passkey(
                            R1PublicKey::from_bytes(&bytes.try_into().map_err(|_| invalid())?)
                                .map_err(|_| invalid())?,
                        ),
                    }
                }
                MemberFile::Vault { launcher_id } => {
                    VaultMember::singleton(parse_bytes32(&launcher_id)?)
                }
            })
        })
        .collect::<Result<Vec<_>, VaultConfigFileError>>()?;

    Ok(VaultSigners::new(members, threshold))
}

fn write_members(members: &[VaultMember]) -> Result<Vec<MemberFile>, VaultConfigFileError> {
    members
        .iter()
        .map(|member| {
            let (public_key, curve) = match member {
                VaultMember::Bls { public_key, .. } => {
                    (hex_string(&public_key.to_bytes()), Curve::Bls12381)
                }
                VaultMember::Secp256k1 { public_key, .. } => {
                    (hex_string(&public_key.to_bytes()), Curve::Secp256k1)
                }
                VaultMember::Secp256r1 { public_key, .. } => {
                    (hex_string(&public_key.to_bytes()), Curve::Secp256r1)
                }
                VaultMember::Passkey { public_key, .. } => {
                    (hex_string(&public_key.to_bytes()), Curve::Webauthn)
                }
                VaultMember::Singleton { launcher_id, .. } => {
                    return Ok(MemberFile::Vault {
                        launcher_id: hex_string(launcher_id),
                    });
                }
                VaultMember::Custom { .. } => return Err(VaultConfigFileError::CustomMember),
            };

            Ok(MemberFile::PublicKey {
                public_key,
                curve,
                key_type: None,
            })
        })
        .collect()
}

/// Cloud Wallet writes hex values with a `0x` prefix.
fn hex_string(bytes: &[u8]) -> String {
    format!("0x{}", hex::encode(bytes))
}

fn parse_hex(value: &str) -> Result<Vec<u8>, VaultConfigFileError> {
    let stripped = value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
        .unwrap_or(value);

    hex::decode(stripped).map_err(|_| VaultConfigFileError::Hex(value.to_string()))
}

fn parse_bytes32(value: &str) -> Result<Bytes32, VaultConfigFileError> {
    let bytes: [u8; 32] = parse_hex(value)?
        .try_into()
        .map_err(|_| VaultConfigFileError::Hex(value.to_string()))?;

    Ok(bytes.into())
}

#[cfg(test)]
mod tests {
    use chia_wallet_sdk::test::{BlsPair, K1Pair, R1Pair};

    use super::*;

    fn config(version: VaultVersion) -> VaultConfig {
        VaultConfig {
            custody: VaultSigners::new(
                vec![
                    VaultMember::secp256r1(R1Pair::new(1).pk),
                    VaultMember::passkey(R1Pair::new(2).pk),
                    VaultMember::secp256k1(K1Pair::new(3).pk),
                ],
                2,
            ),
            recovery: VaultSigners::new(
                vec![
                    VaultMember::bls(BlsPair::new(4).pk),
                    VaultMember::singleton(Bytes32::new([5; 32])),
                ],
                1,
            ),
            recovery_timelock: 86_400,
            version,
        }
    }

    #[test]
    fn test_round_trip() -> anyhow::Result<()> {
        let launcher_id = Bytes32::new([1; 32]);

        for version in [VaultVersion::Legacy, VaultVersion::ForceSingletonRecreation] {
            let config = config(version);
            let json = vault_config_file(launcher_id, &config)?;
            let parsed = parse_vault_config_file(&json)?;

            assert_eq!(parsed.launcher_id, launcher_id);
            assert_eq!(parsed.candidates, [config]);
        }

        Ok(())
    }

    #[test]
    fn test_cloud_wallet_format() -> anyhow::Result<()> {
        let config = config(VaultVersion::ForceSingletonRecreation);
        let json = vault_config_file(Bytes32::new([1; 32]), &config)?;
        let value: serde_json::Value = serde_json::from_str(&json)?;

        assert_eq!(value["vaultPuzzleVersion"], "FORCE_SINGLETON_RECREATION");
        assert_eq!(value["custody"]["threshold"], 2);
        assert_eq!(value["custody"]["members"][0]["type"], "publicKey");
        assert_eq!(value["custody"]["members"][0]["curve"], "SECP256R1");
        assert_eq!(value["custody"]["members"][1]["curve"], "WEBAUTHN");
        assert_eq!(value["custody"]["members"][2]["curve"], "SECP256K1");
        assert_eq!(value["recovery"]["clawbackTimelock"], 86_400);
        assert_eq!(value["recovery"]["members"][0]["curve"], "BLS12_381");
        assert_eq!(value["recovery"]["members"][1]["type"], "vault");
        assert!(
            value["recovery"]["members"][1]["launcherId"]
                .as_str()
                .unwrap()
                .starts_with("0x05")
        );

        Ok(())
    }

    #[test]
    fn test_versions() -> anyhow::Result<()> {
        let json = vault_config_file(
            Bytes32::new([1; 32]),
            &config(VaultVersion::ForceSingletonRecreation),
        )?;

        // Older files write the version as a number, and some leave it out.
        let numbered = json.replace("\"FORCE_SINGLETON_RECREATION\"", "1");
        assert_eq!(
            parse_vault_config_file(&numbered)?.candidates[0].version,
            VaultVersion::Legacy
        );

        let mut value: serde_json::Value = serde_json::from_str(&json)?;
        value.as_object_mut().unwrap().remove("vaultPuzzleVersion");
        let parsed = parse_vault_config_file(&value.to_string())?;
        assert_eq!(
            parsed
                .candidates
                .iter()
                .map(|config| config.version)
                .collect::<Vec<_>>(),
            [VaultVersion::Legacy, VaultVersion::ForceSingletonRecreation]
        );

        Ok(())
    }

    #[test]
    fn test_invalid_files() {
        let json = vault_config_file(
            Bytes32::new([1; 32]),
            &config(VaultVersion::ForceSingletonRecreation),
        )
        .unwrap();

        for invalid in [
            "not json".to_string(),
            json.replace("\"threshold\": 2", "\"threshold\": 4"),
            json.replace("\"threshold\": 2", "\"threshold\": 0"),
            json.replace("\"clawbackTimelock\": 86400", "\"clawbackTimelock\": 0"),
            json.replace("SECP256R1", "ED25519"),
            json.replace("\"0x01", "\"0xzz"),
        ] {
            assert!(parse_vault_config_file(&invalid).is_err(), "{invalid}");
        }

        let custom = VaultConfig {
            custody: VaultSigners::new(
                vec![VaultMember::Custom {
                    puzzle_hash: Bytes32::default(),
                }],
                1,
            ),
            ..config(VaultVersion::Legacy)
        };
        assert!(matches!(
            vault_config_file(Bytes32::default(), &custom),
            Err(VaultConfigFileError::CustomMember)
        ));
    }
}
