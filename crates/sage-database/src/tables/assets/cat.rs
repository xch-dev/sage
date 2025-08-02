use sqlx::query;

use crate::{Asset, AssetKind, Convert, Database, Result};

impl Database {
    pub async fn all_cats(&self) -> Result<Vec<Asset>> {
        query!(
            "
            SELECT
                hash, name, icon_url, description, ticker, precision,
                is_visible, is_sensitive_content, hidden_puzzle_hash
            FROM assets
            WHERE assets.kind = 0 AND assets.id != 0
            ORDER BY name ASC
            "
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|row| {
            Ok(Asset {
                hash: row.hash.convert()?,
                name: row.name,
                ticker: row.ticker,
                precision: row.precision.convert()?,
                icon_url: row.icon_url,
                description: row.description,
                is_visible: row.is_visible,
                is_sensitive_content: row.is_sensitive_content,
                hidden_puzzle_hash: row.hidden_puzzle_hash.convert()?,
                kind: AssetKind::Token,
            })
        })
        .collect()
    }

    pub async fn owned_cats(&self) -> Result<Vec<Asset>> {
        query!(
            "
            SELECT
                hash, name, icon_url, description, ticker, precision,
                is_visible, is_sensitive_content, hidden_puzzle_hash
            FROM assets
            WHERE assets.kind = 0 AND assets.id != 0
            AND EXISTS (
                SELECT 1 FROM coins
                INNER JOIN p2_puzzles ON p2_puzzles.id = coins.p2_puzzle_id
                WHERE coins.asset_id = assets.id
            )
            ORDER BY name ASC
            "
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|row| {
            Ok(Asset {
                hash: row.hash.convert()?,
                name: row.name,
                ticker: row.ticker,
                precision: row.precision.convert()?,
                icon_url: row.icon_url,
                description: row.description,
                is_visible: row.is_visible,
                is_sensitive_content: row.is_sensitive_content,
                hidden_puzzle_hash: row.hidden_puzzle_hash.convert()?,
                kind: AssetKind::Token,
            })
        })
        .collect()
    }
}
