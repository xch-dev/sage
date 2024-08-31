use chia::{
    protocol::{Bytes32, Program},
    puzzles::{LineageProof, Proof},
};
use chia_wallet_sdk::{Nft, NftInfo};
use sqlx::SqliteExecutor;

use crate::{error::Result, to_bytes32, to_coin, to_lineage_proof, Database, DatabaseTx};

impl Database {
    pub async fn insert_nft_coin(
        &self,
        coin_id: Bytes32,
        lineage_proof: LineageProof,
        nft_info: NftInfo<Program>,
    ) -> Result<()> {
        insert_nft_coin(&self.pool, coin_id, lineage_proof, nft_info).await
    }

    pub async fn nft_coin(&self, coin_id: Bytes32) -> Result<Option<Nft<Program>>> {
        nft_coin(&self.pool, coin_id).await
    }
}

impl<'a> DatabaseTx<'a> {
    pub async fn insert_nft_coin(
        &mut self,
        coin_id: Bytes32,
        lineage_proof: LineageProof,
        nft_info: NftInfo<Program>,
    ) -> Result<()> {
        insert_nft_coin(&mut *self.tx, coin_id, lineage_proof, nft_info).await
    }

    pub async fn nft_coin(&mut self, coin_id: Bytes32) -> Result<Option<Nft<Program>>> {
        nft_coin(&mut *self.tx, coin_id).await
    }
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

async fn nft_coin(conn: impl SqliteExecutor<'_>, coin_id: Bytes32) -> Result<Option<Nft<Program>>> {
    let coin_id = coin_id.as_ref();

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
        WHERE cs.coin_id = ?
        ",
        coin_id
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
