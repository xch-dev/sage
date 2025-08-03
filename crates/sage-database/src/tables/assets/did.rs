use chia::protocol::{Bytes32, Coin, Program};
use sqlx::query;

use crate::{Asset, AssetKind, CoinKind, CoinRow, Convert, Database, DatabaseTx, Result};

#[derive(Debug, Clone)]
pub struct DidCoinInfo {
    pub metadata: Program,
    pub recovery_list_hash: Option<Bytes32>,
    pub num_verifications_required: u64,
}

#[derive(Debug, Clone)]
pub struct DidRow {
    pub asset: Asset,
    pub did_info: DidCoinInfo,
    pub coin_row: CoinRow,
}

impl Database {
    pub async fn owned_did(&self, launcher_id: String) -> Result<Option<DidRow>> {
        let hash = launcher_id.as_ref();

        query!(
            "
            SELECT
                asset_hash, asset_name, asset_ticker, asset_precision, asset_icon_url,
                asset_description, asset_is_visible, asset_is_sensitive_content,
                asset_hidden_puzzle_hash, owned_coins.created_height, spent_height,
                parent_coin_hash, puzzle_hash, amount, p2_puzzle_hash,
                metadata, recovery_list_hash, num_verifications_required,
                offer_hash AS 'offer_hash?', created_timestamp, spent_timestamp,
                clawback_expiration_seconds AS 'clawback_timestamp?'
            FROM owned_coins
            INNER JOIN dids ON dids.asset_id = owned_coins.asset_id
            WHERE owned_coins.asset_hash = ?
            ",
            hash
        )
        .fetch_optional(&self.pool)
        .await?
        .map(|row| {
            Ok(DidRow {
                asset: Asset {
                    hash: row.asset_hash.convert()?,
                    name: row.asset_name,
                    ticker: row.asset_ticker,
                    precision: row.asset_precision.convert()?,
                    icon_url: row.asset_icon_url,
                    description: row.asset_description,
                    is_visible: row.asset_is_visible,
                    is_sensitive_content: row.asset_is_sensitive_content,
                    hidden_puzzle_hash: row.asset_hidden_puzzle_hash.convert()?,
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
                    mempool_item_hash: None,
                    offer_hash: row.offer_hash.convert()?,
                    clawback_timestamp: row.clawback_timestamp.map(|t| t as u64),
                    created_height: row.created_height.convert()?,
                    spent_height: row.spent_height.convert()?,
                    created_timestamp: row.created_timestamp.convert()?,
                    spent_timestamp: row.spent_timestamp.convert()?,
                },
            })
        })
        .transpose()?
    }

    pub async fn owned_dids(&self) -> Result<Vec<DidRow>> {
        query!(
            "
            SELECT
                asset_hash, asset_name, asset_ticker, asset_precision, asset_icon_url,
                asset_description, asset_is_visible, asset_is_sensitive_content,
                asset_hidden_puzzle_hash, owned_coins.created_height, spent_height,
                parent_coin_hash, puzzle_hash, amount, p2_puzzle_hash,
                metadata, recovery_list_hash, num_verifications_required,
                offer_hash AS 'offer_hash?', created_timestamp, spent_timestamp,
                clawback_expiration_seconds AS 'clawback_timestamp?'
            FROM owned_coins
            INNER JOIN dids ON dids.asset_id = owned_coins.asset_id
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
                    ticker: row.asset_ticker,
                    precision: row.asset_precision.convert()?,
                    icon_url: row.asset_icon_url,
                    description: row.asset_description,
                    is_visible: row.asset_is_visible,
                    is_sensitive_content: row.asset_is_sensitive_content,
                    hidden_puzzle_hash: row.asset_hidden_puzzle_hash.convert()?,
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
                    mempool_item_hash: None,
                    offer_hash: row.offer_hash.convert()?,
                    clawback_timestamp: row.clawback_timestamp.map(|t| t as u64),
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

    pub async fn update_did(&mut self, hash: Bytes32, coin_info: &DidCoinInfo) -> Result<()> {
        let hash = hash.as_ref();
        let metadata = coin_info.metadata.as_slice();
        let recovery_list_hash = coin_info.recovery_list_hash.as_deref();
        let num_verifications_required: i64 = coin_info.num_verifications_required.try_into()?;

        query!(
            "
            UPDATE dids
            SET
                metadata = ?,
                recovery_list_hash = ?,
                num_verifications_required = ?
            WHERE asset_id = (SELECT id FROM assets WHERE hash = ?)
            ",
            metadata,
            recovery_list_hash,
            num_verifications_required,
            hash
        )
        .execute(&mut *self.tx)
        .await?;

        Ok(())
    }
}
