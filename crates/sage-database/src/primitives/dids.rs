use chia::{
    protocol::{Bytes32, Program},
    puzzles::{LineageProof, Proof},
};
use chia_wallet_sdk::{Did, DidInfo};
use sqlx::SqliteExecutor;

use crate::{to_bytes, to_bytes32, Database, DatabaseTx, Result};

impl Database {
    pub async fn insert_new_did(
        &self,
        launcher_id: Bytes32,
        name: Option<String>,
        visible: bool,
    ) -> Result<()> {
        insert_new_did(&self.pool, launcher_id, name, visible).await
    }

    pub async fn update_did(
        &self,
        launcher_id: Bytes32,
        name: Option<String>,
        visible: bool,
    ) -> Result<()> {
        update_did(&self.pool, launcher_id, name, visible).await
    }

    pub async fn did_coins(&self) -> Result<Vec<DidRow>> {
        did_coins(&self.pool).await
    }

    pub async fn did(&self, did_id: Bytes32) -> Result<Option<Did<Program>>> {
        did(&self.pool, did_id).await
    }

    pub async fn did_name(&self, launcher_id: Bytes32) -> Result<Option<String>> {
        did_name(&self.pool, launcher_id).await
    }
}

impl<'a> DatabaseTx<'a> {
    pub async fn insert_new_did(
        &mut self,
        launcher_id: Bytes32,
        name: Option<String>,
        visible: bool,
    ) -> Result<()> {
        insert_new_did(&mut *self.tx, launcher_id, name, visible).await
    }

    pub async fn insert_did_coin(
        &mut self,
        coin_id: Bytes32,
        lineage_proof: LineageProof,
        did_info: DidInfo<Program>,
    ) -> Result<()> {
        insert_did_coin(&mut *self.tx, coin_id, lineage_proof, did_info).await
    }
}

async fn insert_new_did(
    conn: impl SqliteExecutor<'_>,
    launcher_id: Bytes32,
    name: Option<String>,
    visible: bool,
) -> Result<()> {
    let launcher_id = launcher_id.as_ref();

    sqlx::query!(
        "
        INSERT OR IGNORE INTO `dids` (
            `launcher_id`,
            `name`,
            `visible`
        )
        VALUES (?, ?, ?)
        ",
        launcher_id,
        name,
        visible
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn update_did(
    conn: impl SqliteExecutor<'_>,
    launcher_id: Bytes32,
    name: Option<String>,
    visible: bool,
) -> Result<()> {
    let launcher_id = launcher_id.as_ref();

    sqlx::query!(
        "
        REPLACE INTO `dids` (
            `launcher_id`,
            `name`,
            `visible`
        )
        VALUES (?, ?, ?)
        ",
        launcher_id,
        name,
        visible
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn insert_did_coin(
    conn: impl SqliteExecutor<'_>,
    coin_id: Bytes32,
    lineage_proof: LineageProof,
    did_info: DidInfo<Program>,
) -> Result<()> {
    let coin_id = coin_id.as_ref();
    let parent_parent_coin_id = lineage_proof.parent_parent_coin_info.as_ref();
    let parent_inner_puzzle_hash = lineage_proof.parent_inner_puzzle_hash.as_ref();
    let parent_amount = lineage_proof.parent_amount.to_be_bytes();
    let parent_amount = parent_amount.as_ref();
    let launcher_id = did_info.launcher_id.as_ref();
    let recovery_list_hash = did_info.recovery_list_hash.as_deref();
    let num_verifications_required = did_info.num_verifications_required.to_be_bytes();
    let num_verifications_required = num_verifications_required.as_ref();
    let metadata = did_info.metadata.as_ref();
    let p2_puzzle_hash = did_info.p2_puzzle_hash.as_ref();

    sqlx::query!(
        "
        REPLACE INTO `did_coins` (
            `coin_id`,
            `parent_parent_coin_id`,
            `parent_inner_puzzle_hash`,
            `parent_amount`,
            `launcher_id`,
            `recovery_list_hash`,
            `num_verifications_required`,
            `metadata`,
            `p2_puzzle_hash`
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        ",
        coin_id,
        parent_parent_coin_id,
        parent_inner_puzzle_hash,
        parent_amount,
        launcher_id,
        recovery_list_hash,
        num_verifications_required,
        metadata,
        p2_puzzle_hash
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn did_coins(conn: impl SqliteExecutor<'_>) -> Result<Vec<DidRow>> {
    let rows = sqlx::query!(
        "
        SELECT
            cs.parent_coin_id, cs.puzzle_hash, cs.amount,
            cs.transaction_id AS create_transaction_id, cs.created_height,
            did.parent_parent_coin_id, did.parent_inner_puzzle_hash, did.parent_amount,
            did.launcher_id, did.recovery_list_hash, did.num_verifications_required,
            did.metadata, did.p2_puzzle_hash, name, visible
        FROM `coin_states` AS cs
        INNER JOIN `did_coins` AS did ON cs.coin_id = did.coin_id
        INNER JOIN `dids` ON did.launcher_id = dids.launcher_id
        LEFT JOIN `transaction_spends` ON cs.coin_id = transaction_spends.coin_id
        WHERE cs.spent_height IS NULL
        AND transaction_spends.transaction_id IS NULL
        "
    )
    .fetch_all(conn)
    .await?;

    rows.into_iter()
        .map(|row| {
            Ok(DidRow {
                did: Did {
                    coin: to_coin(&row.parent_coin_id, &row.puzzle_hash, &row.amount)?,
                    proof: Proof::Lineage(to_lineage_proof(
                        &row.parent_parent_coin_id,
                        &row.parent_inner_puzzle_hash,
                        &row.parent_amount,
                    )?),
                    info: DidInfo::<Program> {
                        launcher_id: to_bytes32(&row.launcher_id)?,
                        recovery_list_hash: row
                            .recovery_list_hash
                            .map(|hash| to_bytes32(&hash))
                            .transpose()?,
                        num_verifications_required: u64::from_be_bytes(to_bytes(
                            &row.num_verifications_required,
                        )?),
                        metadata: row.metadata.into(),
                        p2_puzzle_hash: to_bytes32(&row.p2_puzzle_hash)?,
                    },
                },
                name: row.name,
                visible: row.visible,
                create_transaction_id: row
                    .create_transaction_id
                    .as_deref()
                    .map(to_bytes32)
                    .transpose()?,
                created_height: row.created_height.map(TryInto::try_into).transpose()?,
            })
        })
        .collect()
}

async fn did(conn: impl SqliteExecutor<'_>, did_id: Bytes32) -> Result<Option<Did<Program>>> {
    let did_id = did_id.as_ref();

    let Some(row) = sqlx::query!(
        "
        SELECT
            cs.parent_coin_id, cs.puzzle_hash, cs.amount,
            did.parent_parent_coin_id, did.parent_inner_puzzle_hash, did.parent_amount,
            did.launcher_id, did.recovery_list_hash, did.num_verifications_required,
            did.metadata, did.p2_puzzle_hash
        FROM `coin_states` AS cs
        INNER JOIN `did_coins` AS did ON cs.coin_id = did.coin_id
        LEFT JOIN `transaction_spends` ON cs.coin_id = transaction_spends.coin_id
        WHERE did.launcher_id = ?
        AND cs.spent_height IS NULL
        AND cs.created_height IS NOT NULL
        AND cs.transaction_id IS NULL
        AND transaction_spends.transaction_id IS NULL
        ",
        did_id
    )
    .fetch_optional(conn)
    .await?
    else {
        return Ok(None);
    };

    Ok(Some(Did {
        coin: to_coin(&row.parent_coin_id, &row.puzzle_hash, &row.amount)?,
        proof: Proof::Lineage(to_lineage_proof(
            &row.parent_parent_coin_id,
            &row.parent_inner_puzzle_hash,
            &row.parent_amount,
        )?),
        info: DidInfo::<Program> {
            launcher_id: to_bytes32(&row.launcher_id)?,
            recovery_list_hash: row
                .recovery_list_hash
                .map(|hash| to_bytes32(&hash))
                .transpose()?,
            num_verifications_required: u64::from_be_bytes(to_bytes(
                &row.num_verifications_required,
            )?),
            metadata: row.metadata.into(),
            p2_puzzle_hash: to_bytes32(&row.p2_puzzle_hash)?,
        },
    }))
}

async fn did_name(conn: impl SqliteExecutor<'_>, launcher_id: Bytes32) -> Result<Option<String>> {
    let launcher_id = launcher_id.as_ref();

    let Some(row) = sqlx::query!(
        "
        SELECT `name`
        FROM `dids`
        WHERE `launcher_id` = ?
        ",
        launcher_id
    )
    .fetch_optional(conn)
    .await?
    else {
        return Ok(None);
    };

    Ok(row.name)
}
