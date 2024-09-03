use chia::{
    protocol::{Bytes32, Program},
    puzzles::{LineageProof, Proof},
};
use chia_wallet_sdk::{Cat, Did, DidInfo, Nft, NftInfo};
use sqlx::SqliteExecutor;

use crate::{error::Result, to_bytes, to_bytes32, to_coin, to_lineage_proof, Database, DatabaseTx};

impl Database {
    pub async fn insert_cat_coin(
        &self,
        coin_id: Bytes32,
        lineage_proof: LineageProof,
        p2_puzzle_hash: Bytes32,
        asset_id: Bytes32,
    ) -> Result<()> {
        insert_cat_coin(&self.pool, coin_id, lineage_proof, p2_puzzle_hash, asset_id).await
    }

    pub async fn cat_coin(&self, coin_id: Bytes32) -> Result<Option<Cat>> {
        cat_coin(&self.pool, coin_id).await
    }

    pub async fn insert_nft_coin(
        &self,
        coin_id: Bytes32,
        lineage_proof: LineageProof,
        nft_info: NftInfo<Program>,
    ) -> Result<()> {
        insert_nft_coin(&self.pool, coin_id, lineage_proof, nft_info).await
    }

    pub async fn nft_coin(&self, launcher_id: Bytes32) -> Result<Option<Nft<Program>>> {
        nft_coin(&self.pool, launcher_id).await
    }

    pub async fn nft_coins(&self) -> Result<Vec<Nft<Program>>> {
        nft_coins(&self.pool).await
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

    pub async fn insert_nft_coin(
        &mut self,
        coin_id: Bytes32,
        lineage_proof: LineageProof,
        nft_info: NftInfo<Program>,
    ) -> Result<()> {
        insert_nft_coin(&mut *self.tx, coin_id, lineage_proof, nft_info).await
    }

    pub async fn nft_coin(&mut self, launcher_id: Bytes32) -> Result<Option<Nft<Program>>> {
        nft_coin(&mut *self.tx, launcher_id).await
    }

    pub async fn nft_coins(&mut self) -> Result<Vec<Nft<Program>>> {
        nft_coins(&mut *self.tx).await
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

async fn insert_nft_coin(
    conn: impl SqliteExecutor<'_>,
    coin_id: Bytes32,
    lineage_proof: LineageProof,
    nft_info: NftInfo<Program>,
) -> Result<()> {
    let coin_id = coin_id.as_ref();
    let parent_parent_coin_id = lineage_proof.parent_parent_coin_info.as_ref();
    let parent_inner_puzzle_hash = lineage_proof.parent_inner_puzzle_hash.as_ref();
    let parent_amount = lineage_proof.parent_amount.to_be_bytes();
    let parent_amount = parent_amount.as_ref();
    let launcher_id = nft_info.launcher_id.as_ref();
    let metadata = nft_info.metadata.as_ref();
    let metadata_updater_puzzle_hash = nft_info.metadata_updater_puzzle_hash.as_ref();
    let current_owner = nft_info.current_owner.map(|owner| owner.to_vec());
    let royalty_puzzle_hash = nft_info.royalty_puzzle_hash.as_ref();
    let p2_puzzle_hash = nft_info.p2_puzzle_hash.as_ref();

    sqlx::query!(
        "
        REPLACE INTO `nft_coins` (
            `coin_id`,
            `parent_parent_coin_id`,
            `parent_inner_puzzle_hash`,
            `parent_amount`,
            `launcher_id`,
            `metadata`,
            `metadata_updater_puzzle_hash`,
            `current_owner`,
            `royalty_puzzle_hash`,
            `royalty_ten_thousandths`,
            `p2_puzzle_hash`
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ",
        coin_id,
        parent_parent_coin_id,
        parent_inner_puzzle_hash,
        parent_amount,
        launcher_id,
        metadata,
        metadata_updater_puzzle_hash,
        current_owner,
        royalty_puzzle_hash,
        nft_info.royalty_ten_thousandths,
        p2_puzzle_hash
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn nft_coin(
    conn: impl SqliteExecutor<'_>,
    launcher_id: Bytes32,
) -> Result<Option<Nft<Program>>> {
    let launcher_id = launcher_id.as_ref();

    let Some(row) = sqlx::query!(
        "
        SELECT
            cs.parent_coin_id, cs.puzzle_hash, cs.amount,
            nft.parent_parent_coin_id, nft.parent_inner_puzzle_hash, nft.parent_amount,
            nft.launcher_id, nft.metadata, nft.metadata_updater_puzzle_hash,
            nft.current_owner, nft.royalty_puzzle_hash, nft.royalty_ten_thousandths,
            nft.p2_puzzle_hash
        FROM `coin_states` AS cs
        INNER JOIN `nft_coins` AS nft
        ON cs.coin_id = nft.coin_id
        WHERE nft.launcher_id = ?
        ",
        launcher_id
    )
    .fetch_optional(conn)
    .await?
    else {
        return Ok(None);
    };

    Ok(Some(Nft {
        coin: to_coin(&row.parent_coin_id, &row.puzzle_hash, &row.amount)?,
        proof: Proof::Lineage(to_lineage_proof(
            &row.parent_parent_coin_id,
            &row.parent_inner_puzzle_hash,
            &row.parent_amount,
        )?),
        info: NftInfo {
            launcher_id: to_bytes32(&row.launcher_id)?,
            metadata: row.metadata.into(),
            metadata_updater_puzzle_hash: to_bytes32(&row.metadata_updater_puzzle_hash)?,
            current_owner: row
                .current_owner
                .map(|owner| to_bytes32(&owner))
                .transpose()?,
            royalty_puzzle_hash: to_bytes32(&row.royalty_puzzle_hash)?,
            royalty_ten_thousandths: row.royalty_ten_thousandths.try_into()?,
            p2_puzzle_hash: to_bytes32(&row.p2_puzzle_hash)?,
        },
    }))
}

async fn nft_coins(conn: impl SqliteExecutor<'_>) -> Result<Vec<Nft<Program>>> {
    let rows = sqlx::query!(
        "
        SELECT
            cs.parent_coin_id, cs.puzzle_hash, cs.amount,
            nft.parent_parent_coin_id, nft.parent_inner_puzzle_hash, nft.parent_amount,
            nft.launcher_id, nft.metadata, nft.metadata_updater_puzzle_hash,
            nft.current_owner, nft.royalty_puzzle_hash, nft.royalty_ten_thousandths,
            nft.p2_puzzle_hash
        FROM `coin_states` AS cs
        INNER JOIN `nft_coins` AS nft
        ON cs.coin_id = nft.coin_id
        "
    )
    .fetch_all(conn)
    .await?;

    rows.into_iter()
        .map(|row| {
            Ok(Nft {
                coin: to_coin(&row.parent_coin_id, &row.puzzle_hash, &row.amount)?,
                proof: Proof::Lineage(to_lineage_proof(
                    &row.parent_parent_coin_id,
                    &row.parent_inner_puzzle_hash,
                    &row.parent_amount,
                )?),
                info: NftInfo {
                    launcher_id: to_bytes32(&row.launcher_id)?,
                    metadata: row.metadata.into(),
                    metadata_updater_puzzle_hash: to_bytes32(&row.metadata_updater_puzzle_hash)?,
                    current_owner: row
                        .current_owner
                        .map(|owner| to_bytes32(&owner))
                        .transpose()?,
                    royalty_puzzle_hash: to_bytes32(&row.royalty_puzzle_hash)?,
                    royalty_ten_thousandths: row.royalty_ten_thousandths.try_into()?,
                    p2_puzzle_hash: to_bytes32(&row.p2_puzzle_hash)?,
                },
            })
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
