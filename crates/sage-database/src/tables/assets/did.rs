use chia::protocol::{Bytes32, Coin};
use sqlx::query;

use crate::{
    Asset, AssetKind, CoinKind, CoinRow, Convert, Database, DatabaseTx, DidCoinInfo, Result,
};

#[derive(Debug, Clone)]
pub struct DidRow {
    pub asset: Asset,
    pub did_info: DidCoinInfo,
    pub coin_row: CoinRow,
}

impl Database {
    pub async fn owned_dids(&self) -> Result<Vec<DidRow>> {
        query!(
            "
            SELECT
                asset_hash, asset_name, asset_icon_url, asset_description,
                asset_is_visible, asset_is_sensitive_content,
                owned_coins.created_height, spent_height,
                parent_coin_hash, puzzle_hash, amount, p2_puzzle_hash,
                metadata, recovery_list_hash, num_verifications_required,
                (
                    SELECT hash FROM offers
                    INNER JOIN offer_coins ON offer_coins.offer_id = offers.id
                    WHERE offer_coins.coin_id = owned_coins.coin_id
                    AND offers.status <= 1
                    LIMIT 1
                ) AS offer_id,
                (
                    SELECT timestamp FROM blocks
                    WHERE height = owned_coins.created_height
                ) AS created_timestamp,
                (
                    SELECT timestamp FROM blocks
                    WHERE height = owned_coins.spent_height
                ) AS spent_timestamp
            FROM owned_coins
            INNER JOIN dids ON dids.asset_id = owned_coins.asset_id
            WHERE asset_kind = 2
            ORDER BY asset_name ASC
            "
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|row| {
            Ok(DidRow {
                asset: Asset {
                    hash: row.asset_hash.convert()?,
                    name: row.asset_name,
                    icon_url: row.asset_icon_url,
                    description: row.asset_description,
                    is_visible: row.asset_is_visible,
                    is_sensitive_content: row.asset_is_sensitive_content,
                    kind: AssetKind::Did,
                },
                did_info: DidCoinInfo {
                    metadata: row.metadata.into(),
                    recovery_list_hash: row.recovery_list_hash.convert()?,
                    num_verifications_required: row.num_verifications_required.convert()?,
                },
                coin_row: CoinRow {
                    coin: Coin::new(
                        row.parent_coin_hash.convert()?,
                        row.puzzle_hash.convert()?,
                        row.amount.convert()?,
                    ),
                    p2_puzzle_hash: row.p2_puzzle_hash.convert()?,
                    kind: CoinKind::Did,
                    transaction_id: None,
                    offer_id: row.offer_id.convert()?,
                    created_height: row.created_height.convert()?,
                    spent_height: row.spent_height.convert()?,
                    created_timestamp: row.created_timestamp.convert()?,
                    spent_timestamp: row.spent_timestamp.convert()?,
                },
            })
        })
        .collect()
    }
}

impl DatabaseTx<'_> {
    pub async fn insert_did(&mut self, hash: Bytes32, coin_info: &DidCoinInfo) -> Result<()> {
        let hash = hash.as_ref();
        let metadata = coin_info.metadata.as_slice();
        let recovery_list_hash = coin_info.recovery_list_hash.as_deref();
        let num_verifications_required: i64 = coin_info.num_verifications_required.try_into()?;

        query!(
            "
            INSERT OR IGNORE INTO dids (
                asset_id, metadata, recovery_list_hash, num_verifications_required
            )
            VALUES ((SELECT id FROM assets WHERE hash = ?), ?, ?, ?)
            ",
            hash,
            metadata,
            recovery_list_hash,
            num_verifications_required
        )
        .execute(&mut *self.tx)
        .await?;

        Ok(())
    }
}
