use chia::{
    protocol::{Bytes32, Coin, CoinState, Program},
    puzzles::{LineageProof, Proof},
};
use chia_wallet_sdk::{Cat, Did, DidInfo};
use sqlx::SqliteExecutor;

use crate::{
    error::Result, to_bytes, to_bytes32, to_coin, to_coin_state, to_lineage_proof, Database,
    DatabaseTx,
};

#[derive(Debug, Clone)]
pub struct CatRow {
    pub asset_id: Bytes32,
    pub name: Option<String>,
    pub ticker: Option<String>,
    pub description: Option<String>,
    pub icon_url: Option<String>,
    pub visible: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct CatCoin {
    pub coin: Coin,
    pub lineage_proof: LineageProof,
    pub p2_puzzle_hash: Bytes32,
}

impl Database {
    pub async fn maybe_insert_cat(&self, row: CatRow) -> Result<()> {
        maybe_insert_cat(&self.pool, row).await
    }

    pub async fn update_cat(&self, row: CatRow) -> Result<()> {
        update_cat(&self.pool, row).await
    }

    pub async fn delete_cat(&self, asset_id: Bytes32) -> Result<()> {
        delete_cat(&self.pool, asset_id).await
    }

    pub async fn cats(&self) -> Result<Vec<CatRow>> {
        cats(&self.pool).await
    }

    pub async fn cat(&self, asset_id: Bytes32) -> Result<Option<CatRow>> {
        cat(&self.pool, asset_id).await
    }

    pub async fn unidentified_cat(&self) -> Result<Option<Bytes32>> {
        unidentified_cat(&self.pool).await
    }

    pub async fn insert_cat_coin(
        &self,
        coin_id: Bytes32,
        lineage_proof: LineageProof,
        p2_puzzle_hash: Bytes32,
        asset_id: Bytes32,
    ) -> Result<()> {
        insert_cat_coin(&self.pool, coin_id, lineage_proof, p2_puzzle_hash, asset_id).await
    }

    pub async fn unspent_cat_coins(&self, asset_id: Bytes32) -> Result<Vec<CatCoin>> {
        unspent_cat_coins(&self.pool, asset_id).await
    }

    pub async fn cat_coin(&self, coin_id: Bytes32) -> Result<Option<Cat>> {
        cat_coin(&self.pool, coin_id).await
    }

    pub async fn cat_coin_states(&self, asset_id: Bytes32) -> Result<Vec<CoinState>> {
        cat_coin_states(&self.pool, asset_id).await
    }

    pub async fn insert_did_coin(
        &self,
        coin_id: Bytes32,
        lineage_proof: LineageProof,
        did_info: DidInfo<Program>,
    ) -> Result<()> {
        insert_did_coin(&self.pool, coin_id, lineage_proof, did_info).await
    }

    pub async fn did_coin(&self, launcher_id: Bytes32) -> Result<Option<Did<Program>>> {
        did_coin(&self.pool, launcher_id).await
    }

    pub async fn did_coins(&self) -> Result<Vec<Did<Program>>> {
        did_coins(&self.pool).await
    }

    pub async fn insert_unknown_coin(&self, coin_id: Bytes32) -> Result<()> {
        insert_unknown_coin(&self.pool, coin_id).await
    }
}

impl<'a> DatabaseTx<'a> {
    pub async fn maybe_insert_cat(&mut self, row: CatRow) -> Result<()> {
        maybe_insert_cat(&mut *self.tx, row).await
    }

    pub async fn update_cat(&mut self, row: CatRow) -> Result<()> {
        update_cat(&mut *self.tx, row).await
    }

    pub async fn delete_cat(&mut self, asset_id: Bytes32) -> Result<()> {
        delete_cat(&mut *self.tx, asset_id).await
    }

    pub async fn cats(&mut self) -> Result<Vec<CatRow>> {
        cats(&mut *self.tx).await
    }

    pub async fn unidentified_cat(&mut self) -> Result<Option<Bytes32>> {
        unidentified_cat(&mut *self.tx).await
    }

    pub async fn insert_cat_coin(
        &mut self,
        coin_id: Bytes32,
        lineage_proof: LineageProof,
        p2_puzzle_hash: Bytes32,
        asset_id: Bytes32,
    ) -> Result<()> {
        insert_cat_coin(
            &mut *self.tx,
            coin_id,
            lineage_proof,
            p2_puzzle_hash,
            asset_id,
        )
        .await
    }

    pub async fn cat_coin(&mut self, coin_id: Bytes32) -> Result<Option<Cat>> {
        cat_coin(&mut *self.tx, coin_id).await
    }

    pub async fn cat_coin_states(&mut self, asset_id: Bytes32) -> Result<Vec<CoinState>> {
        cat_coin_states(&mut *self.tx, asset_id).await
    }

    pub async fn insert_did_coin(
        &mut self,
        coin_id: Bytes32,
        lineage_proof: LineageProof,
        did_info: DidInfo<Program>,
    ) -> Result<()> {
        insert_did_coin(&mut *self.tx, coin_id, lineage_proof, did_info).await
    }

    pub async fn did_coin(&mut self, launcher_id: Bytes32) -> Result<Option<Did<Program>>> {
        did_coin(&mut *self.tx, launcher_id).await
    }

    pub async fn did_coins(&mut self) -> Result<Vec<Did<Program>>> {
        did_coins(&mut *self.tx).await
    }

    pub async fn insert_unknown_coin(&mut self, coin_id: Bytes32) -> Result<()> {
        insert_unknown_coin(&mut *self.tx, coin_id).await
    }
}

async fn maybe_insert_cat(conn: impl SqliteExecutor<'_>, row: CatRow) -> Result<()> {
    let asset_id = row.asset_id.as_ref();

    sqlx::query!(
        "
        INSERT OR IGNORE INTO `cats` (
            `asset_id`,
            `name`,
            `ticker`,
            `description`,
            `icon_url`,
            `visible`
        ) VALUES (?, ?, ?, ?, ?, ?)
        ",
        asset_id,
        row.name,
        row.ticker,
        row.description,
        row.icon_url,
        row.visible,
    )
    .execute(conn)
    .await?;
    Ok(())
}

async fn update_cat(conn: impl SqliteExecutor<'_>, row: CatRow) -> Result<()> {
    let asset_id = row.asset_id.as_ref();

    sqlx::query!(
        "
        REPLACE INTO `cats` (
            `asset_id`,
            `name`,
            `ticker`,
            `description`,
            `icon_url`,
            `visible`
        ) VALUES (?, ?, ?, ?, ?, ?)
        ",
        asset_id,
        row.name,
        row.ticker,
        row.description,
        row.icon_url,
        row.visible
    )
    .execute(conn)
    .await?;
    Ok(())
}

async fn delete_cat(conn: impl SqliteExecutor<'_>, asset_id: Bytes32) -> Result<()> {
    let asset_id = asset_id.as_ref();

    sqlx::query!(
        "
        DELETE FROM `cats` WHERE `asset_id` = ?
        ",
        asset_id
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn cats(conn: impl SqliteExecutor<'_>) -> Result<Vec<CatRow>> {
    let rows = sqlx::query!(
        "
        SELECT
            `asset_id`,
            `name`,
            `ticker`,
            `description`,
            `icon_url`,
            `visible`
        FROM `cats`
        ORDER BY `name` ASC, `asset_id` ASC
        "
    )
    .fetch_all(conn)
    .await?;

    rows.into_iter()
        .map(|row| {
            Ok(CatRow {
                asset_id: to_bytes32(&row.asset_id)?,
                name: row.name,
                ticker: row.ticker,
                description: row.description,
                icon_url: row.icon_url,
                visible: row.visible,
            })
        })
        .collect()
}

async fn cat(conn: impl SqliteExecutor<'_>, asset_id: Bytes32) -> Result<Option<CatRow>> {
    let asset_id = asset_id.as_ref();

    let row = sqlx::query!(
        "
        SELECT
            `asset_id`,
            `name`,
            `ticker`,
            `description`,
            `icon_url`,
            `visible`
        FROM `cats`
        WHERE `asset_id` = ?
        ",
        asset_id
    )
    .fetch_optional(conn)
    .await?;

    row.map(|row| {
        Ok(CatRow {
            asset_id: to_bytes32(&row.asset_id)?,
            name: row.name,
            ticker: row.ticker,
            description: row.description,
            icon_url: row.icon_url,
            visible: row.visible,
        })
    })
    .transpose()
}

async fn unidentified_cat(conn: impl SqliteExecutor<'_>) -> Result<Option<Bytes32>> {
    let rows = sqlx::query!(
        "
        SELECT `asset_id` FROM `cat_coins`
        WHERE `asset_id` NOT IN (SELECT `asset_id` FROM `cats`)
        LIMIT 1
        "
    )
    .fetch_optional(conn)
    .await?;
    rows.map(|row| to_bytes32(&row.asset_id)).transpose()
}

async fn insert_cat_coin(
    conn: impl SqliteExecutor<'_>,
    coin_id: Bytes32,
    lineage_proof: LineageProof,
    p2_puzzle_hash: Bytes32,
    asset_id: Bytes32,
) -> Result<()> {
    let coin_id = coin_id.as_ref();
    let parent_parent_coin_id = lineage_proof.parent_parent_coin_info.as_ref();
    let parent_inner_puzzle_hash = lineage_proof.parent_inner_puzzle_hash.as_ref();
    let parent_amount = lineage_proof.parent_amount.to_be_bytes();
    let parent_amount = parent_amount.as_ref();
    let p2_puzzle_hash = p2_puzzle_hash.as_ref();
    let asset_id = asset_id.as_ref();

    sqlx::query!(
        "
        REPLACE INTO `cat_coins` (
            `coin_id`,
            `parent_parent_coin_id`,
            `parent_inner_puzzle_hash`,
            `parent_amount`,
            `p2_puzzle_hash`,
            `asset_id`
        )
        VALUES (?, ?, ?, ?, ?, ?)
        ",
        coin_id,
        parent_parent_coin_id,
        parent_inner_puzzle_hash,
        parent_amount,
        p2_puzzle_hash,
        asset_id
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn unspent_cat_coins(
    conn: impl SqliteExecutor<'_>,
    asset_id: Bytes32,
) -> Result<Vec<CatCoin>> {
    let asset_id = asset_id.as_ref();

    let rows = sqlx::query!(
        "
        SELECT
            `parent_coin_id`, `puzzle_hash`, `amount`, `p2_puzzle_hash`,
            `parent_parent_coin_id`, `parent_inner_puzzle_hash`, `parent_amount`
        FROM `cat_coins`
        INNER JOIN `coin_states` ON `cat_coins`.`coin_id` = `coin_states`.`coin_id`
        WHERE `cat_coins`.`asset_id` = ? AND `coin_states`.`spent_height` IS NULL
        ",
        asset_id
    )
    .fetch_all(conn)
    .await?;

    rows.into_iter()
        .map(|row| {
            Ok(CatCoin {
                coin: to_coin(&row.parent_coin_id, &row.puzzle_hash, &row.amount)?,
                lineage_proof: to_lineage_proof(
                    &row.parent_parent_coin_id,
                    &row.parent_inner_puzzle_hash,
                    &row.parent_amount,
                )?,
                p2_puzzle_hash: to_bytes32(&row.p2_puzzle_hash)?,
            })
        })
        .collect()
}

async fn cat_coin(conn: impl SqliteExecutor<'_>, coin_id: Bytes32) -> Result<Option<Cat>> {
    let coin_id = coin_id.as_ref();

    let Some(row) = sqlx::query!(
        "
        SELECT
            cs.parent_coin_id, cs.puzzle_hash, cs.amount,
            cat.parent_parent_coin_id, cat.parent_inner_puzzle_hash, cat.parent_amount,
            cat.p2_puzzle_hash, cat.asset_id
        FROM `coin_states` AS cs
        INNER JOIN `cat_coins` AS cat
        ON cs.coin_id = cat.coin_id
        WHERE cs.coin_id = ?
        ",
        coin_id
    )
    .fetch_optional(conn)
    .await?
    else {
        return Ok(None);
    };

    Ok(Some(Cat {
        coin: to_coin(&row.parent_coin_id, &row.puzzle_hash, &row.amount)?,
        lineage_proof: Some(to_lineage_proof(
            &row.parent_parent_coin_id,
            &row.parent_inner_puzzle_hash,
            &row.parent_amount,
        )?),
        p2_puzzle_hash: to_bytes32(&row.p2_puzzle_hash)?,
        asset_id: to_bytes32(&row.asset_id)?,
    }))
}

async fn cat_coin_states(
    conn: impl SqliteExecutor<'_>,
    asset_id: Bytes32,
) -> Result<Vec<CoinState>> {
    let asset_id = asset_id.as_ref();

    let rows = sqlx::query!(
        "
        SELECT
            cs.parent_coin_id, cs.puzzle_hash, cs.amount,
            cs.spent_height, cs.created_height
        FROM `coin_states` AS cs
        INNER JOIN `cat_coins` AS cat
        ON cs.coin_id = cat.coin_id
        WHERE cat.asset_id = ?
        ",
        asset_id
    )
    .fetch_all(conn)
    .await?;

    rows.into_iter()
        .map(|row| {
            to_coin_state(
                to_coin(&row.parent_coin_id, &row.puzzle_hash, &row.amount)?,
                row.created_height,
                row.spent_height,
            )
        })
        .collect()
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

async fn did_coin(
    conn: impl SqliteExecutor<'_>,
    launcher_id: Bytes32,
) -> Result<Option<Did<Program>>> {
    let launcher_id = launcher_id.as_ref();

    let Some(row) = sqlx::query!(
        "
        SELECT
            cs.parent_coin_id, cs.puzzle_hash, cs.amount,
            did.parent_parent_coin_id, did.parent_inner_puzzle_hash, did.parent_amount,
            did.launcher_id, did.recovery_list_hash, did.num_verifications_required,
            did.metadata, did.p2_puzzle_hash
        FROM `coin_states` AS cs
        INNER JOIN `did_coins` AS did
        ON cs.coin_id = did.coin_id
        WHERE did.launcher_id = ?
        ",
        launcher_id
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
        info: DidInfo {
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

async fn did_coins(conn: impl SqliteExecutor<'_>) -> Result<Vec<Did<Program>>> {
    let rows = sqlx::query!(
        "
        SELECT
            cs.parent_coin_id, cs.puzzle_hash, cs.amount,
            did.parent_parent_coin_id, did.parent_inner_puzzle_hash, did.parent_amount,
            did.launcher_id, did.recovery_list_hash, did.num_verifications_required,
            did.metadata, did.p2_puzzle_hash
        FROM `coin_states` AS cs
        INNER JOIN `did_coins` AS did
        ON cs.coin_id = did.coin_id
        WHERE cs.spent_height IS NULL
        "
    )
    .fetch_all(conn)
    .await?;

    rows.into_iter()
        .map(|row| {
            Ok(Did {
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
            })
        })
        .collect()
}

async fn insert_unknown_coin(conn: impl SqliteExecutor<'_>, coin_id: Bytes32) -> Result<()> {
    let coin_id = coin_id.as_ref();

    sqlx::query!(
        "
        REPLACE INTO `unknown_coins` (`coin_id`) VALUES (?)
        ",
        coin_id
    )
    .execute(conn)
    .await?;

    Ok(())
}
