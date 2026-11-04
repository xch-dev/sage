use chia_wallet_sdk::{driver::VaultVersion, prelude::*};
use sqlx::{SqliteExecutor, query};

use crate::{Convert, Database, DatabaseError, DatabaseTx, Result};

/// A member of a vault's custody or recovery path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VaultMember {
    Bls {
        public_key: PublicKey,
        fast_forward: bool,
    },
    Secp256r1 {
        public_key: R1PublicKey,
        fast_forward: bool,
    },
    Secp256k1 {
        public_key: K1PublicKey,
        fast_forward: bool,
    },
    Passkey {
        public_key: R1PublicKey,
        fast_forward: bool,
    },
    /// Another singleton, such as a vault, which approves spends by sending a message.
    Singleton {
        launcher_id: Bytes32,
        fast_forward: bool,
    },
    /// A member whose puzzle isn't known, only its hash.
    Custom { puzzle_hash: Bytes32 },
}

impl VaultMember {
    /// A BLS member, which doesn't fast forward in Cloud Wallet's layout.
    pub fn bls(public_key: PublicKey) -> Self {
        Self::Bls {
            public_key,
            fast_forward: false,
        }
    }

    /// A secp256r1 member, which fast forwards in Cloud Wallet's layout.
    pub fn secp256r1(public_key: R1PublicKey) -> Self {
        Self::Secp256r1 {
            public_key,
            fast_forward: true,
        }
    }

    /// A secp256k1 member, which fast forwards in Cloud Wallet's layout.
    pub fn secp256k1(public_key: K1PublicKey) -> Self {
        Self::Secp256k1 {
            public_key,
            fast_forward: true,
        }
    }

    /// A passkey member, which fast forwards in Cloud Wallet's layout.
    pub fn passkey(public_key: R1PublicKey) -> Self {
        Self::Passkey {
            public_key,
            fast_forward: true,
        }
    }

    /// A singleton member, which doesn't fast forward in Cloud Wallet's layout.
    pub fn singleton(launcher_id: Bytes32) -> Self {
        Self::Singleton {
            launcher_id,
            fast_forward: false,
        }
    }

    fn kind(&self) -> i64 {
        match self {
            Self::Bls { .. } => 0,
            Self::Secp256r1 { .. } => 1,
            Self::Secp256k1 { .. } => 2,
            Self::Passkey { .. } => 3,
            Self::Singleton { .. } => 4,
            Self::Custom { .. } => 5,
        }
    }

    fn key(&self) -> Vec<u8> {
        match self {
            Self::Bls { public_key, .. } => public_key.to_bytes().to_vec(),
            Self::Secp256r1 { public_key, .. } | Self::Passkey { public_key, .. } => {
                public_key.to_bytes().to_vec()
            }
            Self::Secp256k1 { public_key, .. } => public_key.to_bytes().to_vec(),
            Self::Singleton { launcher_id, .. } => launcher_id.to_vec(),
            Self::Custom { puzzle_hash } => puzzle_hash.to_vec(),
        }
    }

    fn fast_forward(&self) -> bool {
        match self {
            Self::Bls { fast_forward, .. }
            | Self::Secp256r1 { fast_forward, .. }
            | Self::Secp256k1 { fast_forward, .. }
            | Self::Passkey { fast_forward, .. }
            | Self::Singleton { fast_forward, .. } => *fast_forward,
            Self::Custom { .. } => false,
        }
    }

    fn from_row(kind: i64, key: Vec<u8>, fast_forward: bool) -> Result<Self> {
        Ok(match kind {
            0 => Self::Bls {
                public_key: key.convert()?,
                fast_forward,
            },
            1 => Self::Secp256r1 {
                public_key: r1_public_key(key)?,
                fast_forward,
            },
            2 => Self::Secp256k1 {
                public_key: K1PublicKey::from_bytes(&key.convert()?)
                    .map_err(|_| DatabaseError::InvalidPublicKey)?,
                fast_forward,
            },
            3 => Self::Passkey {
                public_key: r1_public_key(key)?,
                fast_forward,
            },
            4 => Self::Singleton {
                launcher_id: key.convert()?,
                fast_forward,
            },
            5 => Self::Custom {
                puzzle_hash: key.convert()?,
            },
            _ => return Err(DatabaseError::InvalidEnumVariant),
        })
    }
}

fn r1_public_key(key: Vec<u8>) -> Result<R1PublicKey> {
    R1PublicKey::from_bytes(&key.convert()?).map_err(|_| DatabaseError::InvalidPublicKey)
}

/// The members of a vault's custody or recovery path, of which `threshold` must sign.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VaultSigners {
    pub members: Vec<VaultMember>,
    pub threshold: usize,
}

impl VaultSigners {
    pub fn new(members: Vec<VaultMember>, threshold: usize) -> Self {
        Self { members, threshold }
    }
}

/// A vault in its normal state, whose custody can spend it at any time, and whose recovery can
/// start moving it to new custody after a timelock.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VaultConfig {
    pub custody: VaultSigners,
    pub recovery: VaultSigners,
    pub recovery_timelock: u64,
    pub version: VaultVersion,
}

/// A vault whose recovery has started. The current custody can still cancel it, until the timelock
/// expires and anyone can finish it by recreating the vault with `recovery_custody_hash`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VaultRecoveryConfig {
    pub custody: VaultSigners,
    pub recovery_timelock: u64,
    pub version: VaultVersion,
    /// The custody hash of the vault once recovery finishes.
    pub recovery_custody_hash: Bytes32,
    /// The amount of the vault once recovery finishes. Legacy vaults are always recreated with 1.
    pub amount: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VaultLayout {
    Custody(VaultConfig),
    Recovery(VaultRecoveryConfig),
}

impl VaultLayout {
    pub fn version(&self) -> VaultVersion {
        match self {
            Self::Custody(config) => config.version,
            Self::Recovery(config) => config.version,
        }
    }
}

/// Where a vault config came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VaultConfigSource {
    /// Decoded from a CHIP-0043 memo, which was verified against the custody hash.
    Memo,
    /// Rebuilt from what the vault's spends reveal and its recovery key, which was verified against
    /// the custody hash.
    Reconstructed,
    Unknown,
}

impl Convert<VaultConfigSource> for i64 {
    fn convert(self) -> Result<VaultConfigSource> {
        Ok(match self {
            0 => VaultConfigSource::Memo,
            1 => VaultConfigSource::Reconstructed,
            2 => VaultConfigSource::Unknown,
            _ => return Err(DatabaseError::InvalidEnumVariant),
        })
    }
}

fn encode_vault_version(version: VaultVersion) -> i64 {
    match version {
        VaultVersion::Legacy => 0,
        VaultVersion::ForceSingletonRecreation => 1,
    }
}

fn decode_vault_version(version: i64) -> Result<VaultVersion> {
    Ok(match version {
        0 => VaultVersion::Legacy,
        1 => VaultVersion::ForceSingletonRecreation,
        _ => return Err(DatabaseError::InvalidEnumVariant),
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VaultConfigRow {
    pub custody_hash: Bytes32,
    pub layout: VaultLayout,
    pub source: VaultConfigSource,
}

impl Database {
    pub async fn vault_config(&self, custody_hash: Bytes32) -> Result<Option<VaultConfigRow>> {
        let mut tx = self.read_tx().await?;
        let row = vault_config(&mut tx, custody_hash).await?;
        tx.commit().await?;
        Ok(row)
    }
}

impl DatabaseTx<'_> {
    pub async fn vault_config(&mut self, custody_hash: Bytes32) -> Result<Option<VaultConfigRow>> {
        vault_config(self, custody_hash).await
    }

    /// Stores a vault config, which must rebuild `custody_hash`. An existing config for the same
    /// custody hash is replaced, since it describes the same puzzle.
    pub async fn insert_vault_config(&mut self, row: &VaultConfigRow) -> Result<()> {
        let custody_hash = row.custody_hash.as_ref();
        let source = row.source as i64;

        let (custody, recovery, recovery_timelock, version, recovery_custody_hash, amount) =
            match &row.layout {
                VaultLayout::Custody(config) => (
                    &config.custody,
                    Some(&config.recovery),
                    config.recovery_timelock,
                    config.version,
                    None,
                    None,
                ),
                VaultLayout::Recovery(config) => (
                    &config.custody,
                    None,
                    config.recovery_timelock,
                    config.version,
                    Some(config.recovery_custody_hash.to_vec()),
                    Some(config.amount.to_be_bytes().to_vec()),
                ),
            };

        let version = encode_vault_version(version);
        let custody_threshold: i64 = custody.threshold.try_into()?;
        let recovery_threshold: Option<i64> = recovery
            .map(|recovery| recovery.threshold.try_into())
            .transpose()?;
        let recovery_timelock: i64 = recovery_timelock.try_into()?;

        query!(
            "
            DELETE FROM vault_configs WHERE custody_hash = ?;

            INSERT INTO vault_configs (
                custody_hash, puzzle_version, custody_threshold, recovery_threshold,
                recovery_timelock, recovery_custody_hash, recovery_amount, source
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?);
            ",
            custody_hash,
            custody_hash,
            version,
            custody_threshold,
            recovery_threshold,
            recovery_timelock,
            recovery_custody_hash,
            amount,
            source,
        )
        .execute(&mut *self.tx)
        .await?;

        // The members are collected first, so that no borrowing iterator is held across an await.
        let members: Vec<(bool, VaultMember)> = custody
            .members
            .iter()
            .map(|&member| (false, member))
            .chain(
                recovery
                    .into_iter()
                    .flat_map(|recovery| recovery.members.iter().map(|&member| (true, member))),
            )
            .collect();

        for (is_recovery, member) in members {
            let kind = member.kind();
            let key = member.key();
            let fast_forward = member.fast_forward();

            query!(
                "
                INSERT INTO vault_config_members (
                    vault_config_id, is_recovery, kind, key, is_fast_forward
                )
                VALUES ((SELECT id FROM vault_configs WHERE custody_hash = ?), ?, ?, ?, ?)
                ",
                custody_hash,
                is_recovery,
                kind,
                key,
                fast_forward,
            )
            .execute(&mut *self.tx)
            .await?;
        }

        Ok(())
    }
}

async fn vault_config(
    tx: &mut DatabaseTx<'_>,
    custody_hash: Bytes32,
) -> Result<Option<VaultConfigRow>> {
    let custody_hash_ref = custody_hash.as_ref();

    let Some(row) = query!(
        "
        SELECT
            id, puzzle_version, custody_threshold, recovery_threshold, recovery_timelock,
            recovery_custody_hash, recovery_amount, source
        FROM vault_configs
        WHERE custody_hash = ?
        ",
        custody_hash_ref
    )
    .fetch_optional(&mut *tx.tx)
    .await?
    else {
        return Ok(None);
    };

    let (custody, recovery) = vault_config_members(&mut *tx.tx, row.id).await?;

    let custody = VaultSigners::new(custody, row.custody_threshold.try_into()?);
    let recovery_timelock = row.recovery_timelock.convert()?;
    let version = decode_vault_version(row.puzzle_version)?;

    let layout = match (
        row.recovery_threshold,
        row.recovery_custody_hash,
        row.recovery_amount,
    ) {
        (Some(recovery_threshold), None, None) => VaultLayout::Custody(VaultConfig {
            custody,
            recovery: VaultSigners::new(recovery, recovery_threshold.try_into()?),
            recovery_timelock,
            version,
        }),
        (None, Some(recovery_custody_hash), Some(amount)) if recovery.is_empty() => {
            VaultLayout::Recovery(VaultRecoveryConfig {
                custody,
                recovery_timelock,
                version,
                recovery_custody_hash: recovery_custody_hash.convert()?,
                amount: amount.convert()?,
            })
        }
        _ => return Err(DatabaseError::InvalidVaultConfig),
    };

    Ok(Some(VaultConfigRow {
        custody_hash,
        layout,
        source: row.source.convert()?,
    }))
}

async fn vault_config_members(
    conn: impl SqliteExecutor<'_>,
    vault_config_id: i64,
) -> Result<(Vec<VaultMember>, Vec<VaultMember>)> {
    let rows = query!(
        "
        SELECT is_recovery, kind, key, is_fast_forward
        FROM vault_config_members
        WHERE vault_config_id = ?
        ORDER BY id ASC
        ",
        vault_config_id
    )
    .fetch_all(conn)
    .await?;

    let mut custody = Vec::new();
    let mut recovery = Vec::new();

    for row in rows {
        let member = VaultMember::from_row(row.kind, row.key, row.is_fast_forward)?;

        if row.is_recovery {
            recovery.push(member);
        } else {
            custody.push(member);
        }
    }

    Ok((custody, recovery))
}
