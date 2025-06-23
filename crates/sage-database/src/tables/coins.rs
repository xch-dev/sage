use chia::{
    protocol::{Bytes32, Coin, CoinState, Program},
    puzzles::{LineageProof, Proof},
};
use chia_wallet_sdk::driver::{
    Cat, CatInfo, Did, DidInfo, Nft, NftInfo, OptionContract, OptionInfo,
};
use sqlx::{query, SqliteExecutor};

use crate::{AssetKind, Convert, Database, DatabaseError, DatabaseTx, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CoinKind {
    Xch,
    Cat,
    Did,
    Option,
    Nft,
}

impl Database {
    pub async fn unsynced_coins(&self, limit: usize) -> Result<Vec<CoinState>> {
        unsynced_coins(&self.pool, limit).await
    }

    pub async fn sync_coin(
        &self,
        id: i64,
        asset_id: i64,
        p2_puzzle_id: i64,
        hidden_puzzle_hash: Option<Bytes32>,
    ) -> Result<()> {
        sync_coin(&self.pool, id, asset_id, p2_puzzle_id, hidden_puzzle_hash).await
    }

    pub async fn subscription_coin_ids(&self) -> Result<Vec<Bytes32>> {
        subscription_coin_ids(&self.pool).await
    }

    pub async fn xch_balance(&self) -> Result<u128> {
        xch_balance(&self.pool).await
    }

    pub async fn cat_balance(&self, asset_id: Bytes32) -> Result<u128> {
        cat_balance(&self.pool, asset_id).await
    }

    pub async fn spendable_xch_balance(&self) -> Result<u128> {
        spendable_xch_balance(&self.pool).await
    }

    pub async fn spendable_cat_balance(&self, asset_id: Bytes32) -> Result<u128> {
        spendable_cat_balance(&self.pool, asset_id).await
    }

    pub async fn spendable_xch_coins(&self) -> Result<Vec<Coin>> {
        spendable_xch_coins(&self.pool).await
    }

    pub async fn spendable_cat_coins(&self, asset_id: Bytes32) -> Result<Vec<Cat>> {
        spendable_cat_coins(&self.pool, asset_id).await
    }

    pub async fn coin_kind(&self, coin_id: Bytes32) -> Result<Option<CoinKind>> {
        coin_kind(&self.pool, coin_id).await
    }

    pub async fn xch_coin(&self, coin_id: Bytes32) -> Result<Option<Coin>> {
        xch_coin(&self.pool, coin_id).await
    }

    pub async fn cat_coin(&self, coin_id: Bytes32) -> Result<Option<Cat>> {
        cat_coin(&self.pool, coin_id).await
    }

    pub async fn did_coin(&self, coin_id: Bytes32) -> Result<Option<Did<Program>>> {
        did_coin(&self.pool, coin_id).await
    }

    pub async fn nft_coin(&self, coin_id: Bytes32) -> Result<Option<Nft<Program>>> {
        nft_coin(&self.pool, coin_id).await
    }

    pub async fn option_coin(&self, coin_id: Bytes32) -> Result<Option<OptionContract>> {
        option_coin(&self.pool, coin_id).await
    }

    pub async fn did(&self, launcher_id: Bytes32) -> Result<Option<Did<Program>>> {
        did(&self.pool, launcher_id).await
    }

    pub async fn nft(&self, launcher_id: Bytes32) -> Result<Option<Nft<Program>>> {
        nft(&self.pool, launcher_id).await
    }

    pub async fn option(&self, launcher_id: Bytes32) -> Result<Option<OptionContract>> {
        option(&self.pool, launcher_id).await
    }
}

impl DatabaseTx<'_> {
    pub async fn coin_id(&mut self, hash: Bytes32) -> Result<i64> {
        coin_id(&mut *self.tx, hash).await
    }

    pub async fn is_latest_singleton_coin(&mut self, hash: Bytes32) -> Result<bool> {
        is_latest_singleton_coin(&mut *self.tx, hash).await
    }

    pub async fn sync_coin(
        &mut self,
        id: i64,
        asset_id: i64,
        p2_puzzle_id: i64,
        hidden_puzzle_hash: Option<Bytes32>,
    ) -> Result<()> {
        sync_coin(
            &mut *self.tx,
            id,
            asset_id,
            p2_puzzle_id,
            hidden_puzzle_hash,
        )
        .await
    }

    pub async fn delete_coin(&mut self, coin_id: Bytes32) -> Result<()> {
        delete_coin(&mut *self.tx, coin_id).await
    }

    pub async fn insert_lineage_proof(
        &mut self,
        coin_id: i64,
        lineage_proof: LineageProof,
    ) -> Result<()> {
        insert_lineage_proof(&mut *self.tx, coin_id, lineage_proof).await
    }
}

async fn unsynced_coins(conn: impl SqliteExecutor<'_>, limit: usize) -> Result<Vec<CoinState>> {
    let limit = i64::try_from(limit)?;

    query!(
        "
        SELECT parent_coin_hash, puzzle_hash, amount, created_height, spent_height
        FROM coins
        WHERE asset_id IS NULL
        LIMIT ?
        ",
        limit
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(|row| {
        Ok(CoinState::new(
            Coin::new(
                row.parent_coin_hash.convert()?,
                row.puzzle_hash.convert()?,
                row.amount.convert()?,
            ),
            row.spent_height.convert()?,
            row.created_height.convert()?,
        ))
    })
    .collect()
}

async fn delete_coin(conn: impl SqliteExecutor<'_>, coin_id: Bytes32) -> Result<()> {
    let coin_id_ref = coin_id.as_ref();

    query!("DELETE FROM coins WHERE hash = ?", coin_id_ref)
        .execute(conn)
        .await?;

    Ok(())
}

async fn coin_id(conn: impl SqliteExecutor<'_>, hash: Bytes32) -> Result<i64> {
    let hash_ref = hash.as_ref();

    Ok(query!("SELECT id FROM coins WHERE hash = ?", hash_ref)
        .fetch_one(conn)
        .await?
        .id)
}

async fn is_latest_singleton_coin(conn: impl SqliteExecutor<'_>, hash: Bytes32) -> Result<bool> {
    let hash_ref = hash.as_ref();

    let rows = query!(
        "SELECT amount FROM coins WHERE parent_coin_hash = ?",
        hash_ref
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(|row| row.amount.convert())
    .collect::<Result<Vec<u64>>>()?;

    Ok(rows.into_iter().all(|amount| amount % 2 == 0))
}

async fn sync_coin(
    conn: impl SqliteExecutor<'_>,
    id: i64,
    asset_id: i64,
    p2_puzzle_id: i64,
    hidden_puzzle_hash: Option<Bytes32>,
) -> Result<()> {
    let hidden_puzzle_hash_ref = hidden_puzzle_hash.as_deref();

    query!(
        "UPDATE coins SET asset_id = ?, p2_puzzle_id = ?, hidden_puzzle_hash = ? WHERE id = ?",
        asset_id,
        p2_puzzle_id,
        hidden_puzzle_hash_ref,
        id,
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn insert_lineage_proof(
    conn: impl SqliteExecutor<'_>,
    coin_id: i64,
    lineage_proof: LineageProof,
) -> Result<()> {
    let parent_parent_coin_hash_ref = lineage_proof.parent_parent_coin_info.as_ref();
    let parent_inner_puzzle_hash_ref = lineage_proof.parent_inner_puzzle_hash.as_ref();
    let parent_amount_ref = lineage_proof.parent_amount.to_be_bytes().to_vec();

    query!(
        "INSERT INTO lineage_proofs (coin_id, parent_parent_coin_hash, parent_inner_puzzle_hash, parent_amount) VALUES (?, ?, ?, ?)",
        coin_id,
        parent_parent_coin_hash_ref,
        parent_inner_puzzle_hash_ref,
        parent_amount_ref,
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn subscription_coin_ids(conn: impl SqliteExecutor<'_>) -> Result<Vec<Bytes32>> {
    query!("SELECT hash FROM coins WHERE asset_id != 0")
        .fetch_all(conn)
        .await?
        .into_iter()
        .map(|row| row.hash.convert())
        .collect()
}

async fn xch_balance(conn: impl SqliteExecutor<'_>) -> Result<u128> {
    query!("SELECT amount FROM owned_coins WHERE owned_coins.asset_id = 0")
        .fetch_all(conn)
        .await?
        .into_iter()
        .map(|row| -> Result<u128> { row.amount.convert() })
        .sum()
}

async fn cat_balance(conn: impl SqliteExecutor<'_>, asset_id: Bytes32) -> Result<u128> {
    let asset_id_ref = asset_id.as_ref();

    query!(
        "
        SELECT amount FROM owned_coins
        INNER JOIN assets ON assets.id = owned_coins.asset_id
        WHERE assets.hash = ?
        ",
        asset_id_ref
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(|row| -> Result<u128> { row.amount.convert() })
    .sum()
}

async fn spendable_xch_balance(conn: impl SqliteExecutor<'_>) -> Result<u128> {
    query!("SELECT amount FROM spendable_coins WHERE asset_id = 0")
        .fetch_all(conn)
        .await?
        .into_iter()
        .map(|row| -> Result<u128> { row.amount.convert() })
        .sum()
}

async fn spendable_cat_balance(conn: impl SqliteExecutor<'_>, asset_id: Bytes32) -> Result<u128> {
    let asset_id_ref = asset_id.as_ref();

    query!(
        "
        SELECT amount FROM spendable_coins
        INNER JOIN assets ON assets.id = asset_id
        WHERE assets.hash = ?
        ",
        asset_id_ref
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(|row| -> Result<u128> { row.amount.convert() })
    .sum()
}

async fn spendable_xch_coins(conn: impl SqliteExecutor<'_>) -> Result<Vec<Coin>> {
    query!(
        "
        SELECT parent_coin_hash, puzzle_hash, amount FROM spendable_coins
        WHERE asset_id = 0
        "
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(|row| {
        Ok(Coin::new(
            row.parent_coin_hash.convert()?,
            row.puzzle_hash.convert()?,
            row.amount.convert()?,
        ))
    })
    .collect()
}

async fn spendable_cat_coins(conn: impl SqliteExecutor<'_>, asset_id: Bytes32) -> Result<Vec<Cat>> {
    let asset_id_ref = asset_id.as_ref();

    query!(
        "
        SELECT
            parent_coin_hash, puzzle_hash, amount, hidden_puzzle_hash, p2_puzzles.hash AS p2_puzzle_hash,
            parent_parent_coin_hash, parent_inner_puzzle_hash, parent_amount
        FROM spendable_coins
        INNER JOIN assets ON assets.id = spendable_coins.asset_id
        INNER JOIN lineage_proofs ON lineage_proofs.coin_id = coin_id
        INNER JOIN p2_puzzles ON p2_puzzles.id = p2_puzzle_id
        WHERE assets.hash = ?
        ",
        asset_id_ref
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(|row| {
        Ok(Cat::new(
            Coin::new(
                row.parent_coin_hash.convert()?,
                row.puzzle_hash.convert()?,
                row.amount.convert()?,
            ),
            Some(LineageProof {
                parent_parent_coin_info: row.parent_parent_coin_hash.convert()?,
                parent_inner_puzzle_hash: row.parent_inner_puzzle_hash.convert()?,
                parent_amount: row.parent_amount.convert()?,
            }),
            CatInfo::new(
                asset_id,
                row.hidden_puzzle_hash.convert()?,
                row.p2_puzzle_hash.convert()?,
            ),
        ))
    })
    .collect()
}

async fn coin_kind(conn: impl SqliteExecutor<'_>, coin_id: Bytes32) -> Result<Option<CoinKind>> {
    let coin_id_ref = coin_id.as_ref();

    let Some(row) = query!(
        "SELECT asset_id, kind FROM coins INNER JOIN assets ON assets.id = asset_id WHERE coins.hash = ?",
        coin_id_ref
    )
    .fetch_optional(conn)
    .await?
    else {
        return Ok(None);
    };

    let Some(asset_id) = row.asset_id else {
        return Err(DatabaseError::InvalidEnumVariant);
    };

    let kind: AssetKind = row.kind.convert()?;

    Ok(Some(match kind {
        AssetKind::Token => {
            if asset_id == 0 {
                CoinKind::Xch
            } else {
                CoinKind::Cat
            }
        }
        AssetKind::Nft => CoinKind::Nft,
        AssetKind::Did => CoinKind::Did,
        AssetKind::Option => CoinKind::Option,
    }))
}

async fn xch_coin(conn: impl SqliteExecutor<'_>, coin_id: Bytes32) -> Result<Option<Coin>> {
    let coin_id_ref = coin_id.as_ref();

    let Some(row) = query!(
        "
        SELECT
            parent_coin_hash, puzzle_hash, amount
        FROM owned_coins
        WHERE owned_coins.hash = ? AND owned_coins.asset_id = 0
        ",
        coin_id_ref
    )
    .fetch_optional(conn)
    .await?
    else {
        return Ok(None);
    };

    Ok(Some(Coin::new(
        row.parent_coin_hash.convert()?,
        row.puzzle_hash.convert()?,
        row.amount.convert()?,
    )))
}

async fn cat_coin(conn: impl SqliteExecutor<'_>, coin_id: Bytes32) -> Result<Option<Cat>> {
    let coin_id_ref = coin_id.as_ref();

    let Some(row) = query!(
        "
        SELECT
            parent_coin_hash, puzzle_hash, amount, hidden_puzzle_hash, p2_puzzles.hash AS p2_puzzle_hash,
            parent_parent_coin_hash, parent_inner_puzzle_hash, parent_amount, assets.hash AS asset_id
        FROM owned_coins
        INNER JOIN assets ON assets.id = owned_coins.asset_id
        INNER JOIN lineage_proofs ON lineage_proofs.coin_id = coin_id
        INNER JOIN p2_puzzles ON p2_puzzles.id = p2_puzzle_id
        WHERE owned_coins.hash = ?
        ",
        coin_id_ref
    )
    .fetch_optional(conn)
    .await?
    else {
        return Ok(None);
    };

    Ok(Some(Cat::new(
        Coin::new(
            row.parent_coin_hash.convert()?,
            row.puzzle_hash.convert()?,
            row.amount.convert()?,
        ),
        Some(LineageProof {
            parent_parent_coin_info: row.parent_parent_coin_hash.convert()?,
            parent_inner_puzzle_hash: row.parent_inner_puzzle_hash.convert()?,
            parent_amount: row.parent_amount.convert()?,
        }),
        CatInfo::new(
            row.asset_id.convert()?,
            row.hidden_puzzle_hash.convert()?,
            row.p2_puzzle_hash.convert()?,
        ),
    )))
}

async fn did_coin(conn: impl SqliteExecutor<'_>, coin_id: Bytes32) -> Result<Option<Did<Program>>> {
    let coin_id_ref = coin_id.as_ref();

    let Some(row) = query!(
        "
        SELECT
            parent_coin_hash, puzzle_hash, amount, p2_puzzles.hash AS p2_puzzle_hash,
            parent_parent_coin_hash, parent_inner_puzzle_hash, parent_amount,
            assets.hash AS launcher_id, recovery_list_hash, num_verifications_required, metadata
        FROM owned_coins
        INNER JOIN assets ON assets.id = owned_coins.asset_id
        INNER JOIN dids ON dids.asset_id = assets.id
        INNER JOIN lineage_proofs ON lineage_proofs.coin_id = coin_id
        INNER JOIN p2_puzzles ON p2_puzzles.id = p2_puzzle_id
        WHERE owned_coins.hash = ?
        ",
        coin_id_ref
    )
    .fetch_optional(conn)
    .await?
    else {
        return Ok(None);
    };

    Ok(Some(Did::new(
        Coin::new(
            row.parent_coin_hash.convert()?,
            row.puzzle_hash.convert()?,
            row.amount.convert()?,
        ),
        Proof::Lineage(LineageProof {
            parent_parent_coin_info: row.parent_parent_coin_hash.convert()?,
            parent_inner_puzzle_hash: row.parent_inner_puzzle_hash.convert()?,
            parent_amount: row.parent_amount.convert()?,
        }),
        DidInfo::new(
            row.launcher_id.convert()?,
            row.recovery_list_hash.convert()?,
            row.num_verifications_required.convert()?,
            row.metadata.into(),
            row.p2_puzzle_hash.convert()?,
        ),
    )))
}

async fn nft_coin(conn: impl SqliteExecutor<'_>, coin_id: Bytes32) -> Result<Option<Nft<Program>>> {
    let coin_id_ref = coin_id.as_ref();

    let Some(row) = query!(
        "
        SELECT
            parent_coin_hash, puzzle_hash, amount, p2_puzzles.hash AS p2_puzzle_hash,
            parent_parent_coin_hash, parent_inner_puzzle_hash, parent_amount,
            assets.hash AS launcher_id, metadata, metadata_updater_puzzle_hash,
            owner_hash, royalty_puzzle_hash, royalty_basis_points
        FROM owned_coins
        INNER JOIN assets ON assets.id = owned_coins.asset_id
        INNER JOIN nfts ON nfts.asset_id = assets.id
        INNER JOIN lineage_proofs ON lineage_proofs.coin_id = coin_id
        INNER JOIN p2_puzzles ON p2_puzzles.id = p2_puzzle_id
        WHERE owned_coins.hash = ?
        ",
        coin_id_ref
    )
    .fetch_optional(conn)
    .await?
    else {
        return Ok(None);
    };

    Ok(Some(Nft::new(
        Coin::new(
            row.parent_coin_hash.convert()?,
            row.puzzle_hash.convert()?,
            row.amount.convert()?,
        ),
        Proof::Lineage(LineageProof {
            parent_parent_coin_info: row.parent_parent_coin_hash.convert()?,
            parent_inner_puzzle_hash: row.parent_inner_puzzle_hash.convert()?,
            parent_amount: row.parent_amount.convert()?,
        }),
        NftInfo::new(
            row.launcher_id.convert()?,
            row.metadata.into(),
            row.metadata_updater_puzzle_hash.convert()?,
            row.owner_hash.convert()?,
            row.royalty_puzzle_hash.convert()?,
            row.royalty_basis_points.convert()?,
            row.p2_puzzle_hash.convert()?,
        ),
    )))
}

async fn option_coin(
    conn: impl SqliteExecutor<'_>,
    coin_id: Bytes32,
) -> Result<Option<OptionContract>> {
    let coin_id_ref = coin_id.as_ref();

    let Some(row) = query!(
        "
        SELECT
            parent_coin_hash, puzzle_hash, amount, p2_puzzles.hash AS p2_puzzle_hash,
            parent_parent_coin_hash, parent_inner_puzzle_hash, parent_amount,
            assets.hash AS launcher_id, underlying_coin_hash, underlying_delegated_puzzle_hash
        FROM owned_coins
        INNER JOIN assets ON assets.id = owned_coins.asset_id
        INNER JOIN options ON options.asset_id = assets.id
        INNER JOIN lineage_proofs ON lineage_proofs.coin_id = coin_id
        INNER JOIN p2_puzzles ON p2_puzzles.id = p2_puzzle_id
        WHERE owned_coins.hash = ?
        ",
        coin_id_ref
    )
    .fetch_optional(conn)
    .await?
    else {
        return Ok(None);
    };

    Ok(Some(OptionContract::new(
        Coin::new(
            row.parent_coin_hash.convert()?,
            row.puzzle_hash.convert()?,
            row.amount.convert()?,
        ),
        Proof::Lineage(LineageProof {
            parent_parent_coin_info: row.parent_parent_coin_hash.convert()?,
            parent_inner_puzzle_hash: row.parent_inner_puzzle_hash.convert()?,
            parent_amount: row.parent_amount.convert()?,
        }),
        OptionInfo::new(
            row.launcher_id.convert()?,
            row.underlying_coin_hash.convert()?,
            row.underlying_delegated_puzzle_hash.convert()?,
            row.p2_puzzle_hash.convert()?,
        ),
    )))
}

async fn did(conn: impl SqliteExecutor<'_>, launcher_id: Bytes32) -> Result<Option<Did<Program>>> {
    let launcher_id_ref = launcher_id.as_ref();

    let Some(row) = query!(
        "
        SELECT
            parent_coin_hash, puzzle_hash, amount, p2_puzzles.hash AS p2_puzzle_hash,
            parent_parent_coin_hash, parent_inner_puzzle_hash, parent_amount,
            assets.hash AS launcher_id, recovery_list_hash, num_verifications_required, metadata
        FROM owned_coins
        INNER JOIN assets ON assets.id = owned_coins.asset_id
        INNER JOIN dids ON dids.asset_id = assets.id
        INNER JOIN lineage_proofs ON lineage_proofs.coin_id = coin_id
        INNER JOIN p2_puzzles ON p2_puzzles.id = p2_puzzle_id
        WHERE assets.hash = ?
        ",
        launcher_id_ref
    )
    .fetch_optional(conn)
    .await?
    else {
        return Ok(None);
    };

    Ok(Some(Did::new(
        Coin::new(
            row.parent_coin_hash.convert()?,
            row.puzzle_hash.convert()?,
            row.amount.convert()?,
        ),
        Proof::Lineage(LineageProof {
            parent_parent_coin_info: row.parent_parent_coin_hash.convert()?,
            parent_inner_puzzle_hash: row.parent_inner_puzzle_hash.convert()?,
            parent_amount: row.parent_amount.convert()?,
        }),
        DidInfo::new(
            row.launcher_id.convert()?,
            row.recovery_list_hash.convert()?,
            row.num_verifications_required.convert()?,
            row.metadata.into(),
            row.p2_puzzle_hash.convert()?,
        ),
    )))
}

async fn nft(conn: impl SqliteExecutor<'_>, launcher_id: Bytes32) -> Result<Option<Nft<Program>>> {
    let launcher_id_ref = launcher_id.as_ref();

    let Some(row) = query!(
        "
        SELECT
            parent_coin_hash, puzzle_hash, amount, p2_puzzles.hash AS p2_puzzle_hash,
            parent_parent_coin_hash, parent_inner_puzzle_hash, parent_amount,
            assets.hash AS launcher_id, metadata, metadata_updater_puzzle_hash,
            owner_hash, royalty_puzzle_hash, royalty_basis_points
        FROM owned_coins
        INNER JOIN assets ON assets.id = owned_coins.asset_id
        INNER JOIN nfts ON nfts.asset_id = assets.id
        INNER JOIN lineage_proofs ON lineage_proofs.coin_id = coin_id
        INNER JOIN p2_puzzles ON p2_puzzles.id = p2_puzzle_id
        WHERE assets.hash = ?
        ",
        launcher_id_ref
    )
    .fetch_optional(conn)
    .await?
    else {
        return Ok(None);
    };

    Ok(Some(Nft::new(
        Coin::new(
            row.parent_coin_hash.convert()?,
            row.puzzle_hash.convert()?,
            row.amount.convert()?,
        ),
        Proof::Lineage(LineageProof {
            parent_parent_coin_info: row.parent_parent_coin_hash.convert()?,
            parent_inner_puzzle_hash: row.parent_inner_puzzle_hash.convert()?,
            parent_amount: row.parent_amount.convert()?,
        }),
        NftInfo::new(
            row.launcher_id.convert()?,
            row.metadata.into(),
            row.metadata_updater_puzzle_hash.convert()?,
            row.owner_hash.convert()?,
            row.royalty_puzzle_hash.convert()?,
            row.royalty_basis_points.convert()?,
            row.p2_puzzle_hash.convert()?,
        ),
    )))
}

async fn option(
    conn: impl SqliteExecutor<'_>,
    launcher_id: Bytes32,
) -> Result<Option<OptionContract>> {
    let launcher_id_ref = launcher_id.as_ref();

    let Some(row) = query!(
        "
        SELECT
            parent_coin_hash, puzzle_hash, amount, p2_puzzles.hash AS p2_puzzle_hash,
            parent_parent_coin_hash, parent_inner_puzzle_hash, parent_amount,
            assets.hash AS launcher_id, underlying_coin_hash, underlying_delegated_puzzle_hash
        FROM owned_coins
        INNER JOIN assets ON assets.id = owned_coins.asset_id
        INNER JOIN options ON options.asset_id = assets.id
        INNER JOIN lineage_proofs ON lineage_proofs.coin_id = coin_id
        INNER JOIN p2_puzzles ON p2_puzzles.id = p2_puzzle_id
        WHERE assets.hash = ?
        ",
        launcher_id_ref
    )
    .fetch_optional(conn)
    .await?
    else {
        return Ok(None);
    };

    Ok(Some(OptionContract::new(
        Coin::new(
            row.parent_coin_hash.convert()?,
            row.puzzle_hash.convert()?,
            row.amount.convert()?,
        ),
        Proof::Lineage(LineageProof {
            parent_parent_coin_info: row.parent_parent_coin_hash.convert()?,
            parent_inner_puzzle_hash: row.parent_inner_puzzle_hash.convert()?,
            parent_amount: row.parent_amount.convert()?,
        }),
        OptionInfo::new(
            row.launcher_id.convert()?,
            row.underlying_coin_hash.convert()?,
            row.underlying_delegated_puzzle_hash.convert()?,
            row.p2_puzzle_hash.convert()?,
        ),
    )))
}
