use chia_wallet_sdk::{chia::puzzle_types::singleton::SingletonArgs, prelude::*};
use sqlx::{SqliteExecutor, query};

use crate::{Asset, AssetKind, Convert, Database, DatabaseTx, Result, VaultConfigRow, VaultLayout};

/// A vault's state, which is derived from its lineage and the config of its current coin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VaultState {
    /// The vault is tracked, but none of its singleton coins have been synced, for example because
    /// its launcher hasn't been spent yet.
    Unsynced,
    Active,
    /// Recovery has been started, and the vault will move to its recovery custody once the
    /// timelock expires, unless the current custody cancels it first.
    RecoveryPending {
        /// When the coin that started recovery was created, once its block's timestamp is known.
        started_at: Option<u64>,
    },
    Melted,
}

/// A coin of a vault singleton's lineage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VaultCoinRow {
    pub coin: Coin,
    /// The singleton's inner puzzle hash, which is only unknown for an unspent eve coin whose
    /// launcher didn't reveal a memo.
    pub custody_hash: Option<Bytes32>,
    pub created_height: Option<u32>,
    pub spent_height: Option<u32>,
    pub created_timestamp: Option<u64>,
    pub is_children_synced: bool,
}

/// A tracked vault, along with its current coin and config.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VaultStatus {
    pub launcher_id: Bytes32,
    /// The latest coin of the vault's lineage. Once the vault is melted, this is the coin that was
    /// melted. Its custody hash is filled in from a matching config, if its own is unknown.
    pub coin: Option<VaultCoinRow>,
    /// The config which rebuilds the current coin's custody hash, if one is known.
    pub config: Option<VaultConfigRow>,
    pub state: VaultState,
}

impl Database {
    /// The launcher ids of every tracked vault.
    pub async fn vault_launcher_ids(&self) -> Result<Vec<Bytes32>> {
        vault_launcher_ids(&self.pool).await
    }

    /// The launcher ids of tracked vaults whose launcher coin hasn't been synced, so they're synced
    /// from the start of the chain.
    pub async fn unsynced_vault_launcher_ids(&self) -> Result<Vec<Bytes32>> {
        unsynced_vault_launcher_ids(&self.pool).await
    }

    pub async fn vault_coins(&self, launcher_id: Bytes32) -> Result<Vec<VaultCoinRow>> {
        vault_coins(&self.pool, launcher_id).await
    }

    /// The vault with the launcher id, if it's tracked.
    pub async fn vault_status(&self, launcher_id: Bytes32) -> Result<Option<VaultStatus>> {
        let mut tx = self.read_tx().await?;
        let status = vault_status(&mut tx, launcher_id).await?;
        tx.commit().await?;
        Ok(status)
    }
}

impl DatabaseTx<'_> {
    pub async fn vault_launcher_ids(&mut self) -> Result<Vec<Bytes32>> {
        vault_launcher_ids(&mut *self.tx).await
    }

    pub async fn is_vault(&mut self, launcher_id: Bytes32) -> Result<bool> {
        is_vault(&mut *self.tx, launcher_id).await
    }

    /// Tracks a vault by its launcher id. Its coins are synced from its launcher.
    pub async fn insert_vault(&mut self, launcher_id: Bytes32) -> Result<()> {
        self.insert_asset(Asset {
            hash: launcher_id,
            name: None,
            ticker: None,
            precision: 1,
            icon_url: None,
            description: None,
            is_sensitive_content: false,
            is_visible: true,
            hidden_puzzle_hash: None,
            kind: AssetKind::Vault,
        })
        .await
    }

    /// The launcher id of the vault which a coin belongs to, if it's a vault coin or launcher.
    pub async fn vault_launcher_id(&mut self, coin_id: Bytes32) -> Result<Option<Bytes32>> {
        vault_launcher_id(&mut *self.tx, coin_id).await
    }

    /// Adds a coin to a vault's lineage, if the coin is known. A known custody hash is never
    /// replaced by an unknown one.
    pub async fn insert_vault_coin(
        &mut self,
        coin_id: Bytes32,
        custody_hash: Option<Bytes32>,
    ) -> Result<()> {
        insert_vault_coin(&mut *self.tx, coin_id, custody_hash).await
    }

    pub async fn vault_status(&mut self, launcher_id: Bytes32) -> Result<Option<VaultStatus>> {
        vault_status(self, launcher_id).await
    }
}

async fn vault_launcher_ids(conn: impl SqliteExecutor<'_>) -> Result<Vec<Bytes32>> {
    query!("SELECT hash FROM assets WHERE kind = 4 ORDER BY id ASC")
        .fetch_all(conn)
        .await?
        .into_iter()
        .map(|row| row.hash.convert())
        .collect()
}

async fn unsynced_vault_launcher_ids(conn: impl SqliteExecutor<'_>) -> Result<Vec<Bytes32>> {
    query!(
        "
        SELECT hash FROM assets
        WHERE kind = 4
        AND NOT EXISTS (SELECT 1 FROM coins WHERE coins.asset_id = assets.id)
        ORDER BY id ASC
        "
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(|row| row.hash.convert())
    .collect()
}

async fn is_vault(conn: impl SqliteExecutor<'_>, launcher_id: Bytes32) -> Result<bool> {
    let launcher_id = launcher_id.as_ref();

    Ok(query!(
        "SELECT COUNT(*) AS count FROM assets WHERE hash = ? AND kind = 4",
        launcher_id
    )
    .fetch_one(conn)
    .await?
    .count
        > 0)
}

async fn vault_launcher_id(
    conn: impl SqliteExecutor<'_>,
    coin_id: Bytes32,
) -> Result<Option<Bytes32>> {
    let coin_id = coin_id.as_ref();

    query!(
        "
        SELECT assets.hash
        FROM coins
        INNER JOIN assets ON assets.id = coins.asset_id
        WHERE coins.hash = ? AND assets.kind = 4
        ",
        coin_id
    )
    .fetch_optional(conn)
    .await?
    .map(|row| row.hash.convert())
    .transpose()
}

async fn insert_vault_coin(
    conn: impl SqliteExecutor<'_>,
    coin_id: Bytes32,
    custody_hash: Option<Bytes32>,
) -> Result<()> {
    let coin_id = coin_id.as_ref();
    let custody_hash = custody_hash.as_ref().map(AsRef::as_ref);

    query!(
        "
        INSERT INTO vault_coins (coin_id, custody_hash)
        SELECT id, ? FROM coins WHERE hash = ?
        ON CONFLICT(coin_id) DO UPDATE SET
            custody_hash = COALESCE(excluded.custody_hash, custody_hash)
        ",
        custody_hash,
        coin_id
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn vault_coins(
    conn: impl SqliteExecutor<'_>,
    launcher_id: Bytes32,
) -> Result<Vec<VaultCoinRow>> {
    let launcher_id = launcher_id.as_ref();

    query!(
        "
        SELECT
            coins.parent_coin_hash, coins.puzzle_hash, coins.amount, coins.created_height,
            coins.spent_height, coins.is_children_synced, vault_coins.custody_hash,
            blocks.timestamp AS created_timestamp
        FROM vault_coins
        INNER JOIN coins ON coins.id = vault_coins.coin_id
        INNER JOIN assets ON assets.id = coins.asset_id
        LEFT JOIN blocks ON blocks.height = coins.created_height
        WHERE assets.hash = ?
        ORDER BY coins.created_height IS NULL ASC, coins.created_height ASC, vault_coins.id ASC
        ",
        launcher_id
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(|row| {
        Ok(VaultCoinRow {
            coin: Coin::new(
                row.parent_coin_hash.convert()?,
                row.puzzle_hash.convert()?,
                row.amount.convert()?,
            ),
            custody_hash: row.custody_hash.convert()?,
            created_height: row.created_height.convert()?,
            spent_height: row.spent_height.convert()?,
            created_timestamp: row.created_timestamp.convert()?,
            is_children_synced: row.is_children_synced,
        })
    })
    .collect()
}

async fn vault_config_custody_hashes(conn: impl SqliteExecutor<'_>) -> Result<Vec<Bytes32>> {
    query!("SELECT custody_hash FROM vault_configs")
        .fetch_all(conn)
        .await?
        .into_iter()
        .map(|row| row.custody_hash.convert())
        .collect()
}

async fn vault_status(
    tx: &mut DatabaseTx<'_>,
    launcher_id: Bytes32,
) -> Result<Option<VaultStatus>> {
    if !is_vault(&mut *tx.tx, launcher_id).await? {
        return Ok(None);
    }

    let coins = vault_coins(&mut *tx.tx, launcher_id).await?;

    // The current coin is the latest confirmed coin that's unspent. A coin whose creation was undone
    // by a reorg isn't confirmed, so its parent is current again.
    let current = coins
        .iter()
        .rposition(|row| row.created_height.is_some() && row.spent_height.is_none());

    let Some(index) = current.or_else(|| coins.len().checked_sub(1)) else {
        return Ok(Some(VaultStatus {
            launcher_id,
            coin: None,
            config: None,
            state: VaultState::Unsynced,
        }));
    };

    let mut coin = coins[index];

    // An unspent eve coin's custody hash isn't revealed without a memo, but a config imported from
    // a file can still be matched against its puzzle hash.
    if coin.custody_hash.is_none() {
        coin.custody_hash = vault_config_custody_hashes(&mut *tx.tx)
            .await?
            .into_iter()
            .find(|&custody_hash| {
                SingletonArgs::curry_tree_hash(launcher_id, custody_hash.into())
                    == coin.coin.puzzle_hash.into()
            });
    }

    let config = match coin.custody_hash {
        Some(custody_hash) => tx.vault_config(custody_hash).await?,
        None => None,
    };

    // A spent coin whose children have been synced, without a singleton child to replace it, was
    // melted. Until its children are synced, it's still treated as the current coin.
    let state = if current.is_none() && coin.spent_height.is_some() && coin.is_children_synced {
        VaultState::Melted
    } else if let Some(VaultConfigRow {
        layout: VaultLayout::Recovery(_),
        ..
    }) = config
    {
        // Recovery started when the first coin of the trailing run with the recovery custody
        // hash was created, since the custody can recreate the vault while recovery is pending.
        let start = coins[..=index]
            .iter()
            .rev()
            .take_while(|row| row.custody_hash == coin.custody_hash)
            .last()
            .unwrap_or(&coin);

        VaultState::RecoveryPending {
            started_at: start.created_timestamp,
        }
    } else {
        VaultState::Active
    };

    Ok(Some(VaultStatus {
        launcher_id,
        coin: Some(coin),
        config,
        state,
    }))
}
