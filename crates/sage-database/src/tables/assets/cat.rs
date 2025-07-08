use chia::protocol::Bytes32;
use sqlx::query;

use crate::{Asset, AssetKind, Convert, Database, DatabaseTx, Result};

#[derive(Debug, Clone)]
pub struct CatAsset {
    pub asset: Asset,
    pub ticker: Option<String>,
    pub precision: u8,
}

impl Database {
    pub async fn update_cat(
        &self,
        hash: Bytes32,
        ticker: Option<String>,
        precision: u8,
    ) -> Result<()> {
        let hash = hash.as_ref();

        query!(
            "
            UPDATE tokens 
            SET ticker = ?, precision = ?
            WHERE asset_id = (SELECT id FROM assets WHERE hash = ?)
            ",
            ticker,
            precision,
            hash
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn cat_asset(&self, asset_id: Bytes32) -> Result<Option<CatAsset>> {
        let asset_id = asset_id.as_ref();

        query!(
            "
            SELECT
                hash, name, icon_url, description, ticker, precision,
                is_visible, is_sensitive_content
            FROM assets
            INNER JOIN tokens ON tokens.asset_id = assets.id
            WHERE assets.id != 0 AND hash = ?
            ",
            asset_id
        )
        .fetch_optional(&self.pool)
        .await?
        .map(|row| {
            Ok(CatAsset {
                asset: Asset {
                    hash: row.hash.convert()?,
                    name: row.name,
                    icon_url: row.icon_url,
                    description: row.description,
                    is_visible: row.is_visible,
                    is_sensitive_content: row.is_sensitive_content,
                    kind: AssetKind::Token,
                },
                ticker: row.ticker,
                precision: row.precision.convert()?,
            })
        })
        .transpose()
    }

    pub async fn owned_cats(&self) -> Result<Vec<CatAsset>> {
        query!(
            "
            SELECT
                hash, name, icon_url, description, ticker, precision,
                is_visible, is_sensitive_content
            FROM assets
            INNER JOIN tokens ON tokens.asset_id = assets.id
            WHERE assets.id != 0
            AND EXISTS (
                SELECT 1
                FROM internal_coins
                WHERE internal_coins.asset_id = assets.id
            )
            ORDER BY name ASC
            "
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|row| {
            Ok(CatAsset {
                asset: Asset {
                    hash: row.hash.convert()?,
                    name: row.name,
                    icon_url: row.icon_url,
                    description: row.description,
                    is_visible: row.is_visible,
                    is_sensitive_content: row.is_sensitive_content,
                    kind: AssetKind::Token,
                },
                ticker: row.ticker,
                precision: row.precision.convert()?,
            })
        })
        .collect()
    }
}

impl DatabaseTx<'_> {
    pub async fn insert_cat(&mut self, cat: CatAsset) -> Result<()> {
        let hash = cat.asset.hash.as_ref();

        let asset_id = query!(
            "
            INSERT INTO assets (hash, kind, name, icon_url, description, is_visible)
            VALUES (?, 0, ?, ?, ?, ?)
            ON CONFLICT(hash) DO UPDATE SET
                name = COALESCE(name, excluded.name),
                icon_url = COALESCE(icon_url, excluded.icon_url),
                description = COALESCE(description, excluded.description),
                is_sensitive_content = is_sensitive_content OR excluded.is_sensitive_content
            RETURNING id
            ",
            hash,
            cat.asset.name,
            cat.asset.icon_url,
            cat.asset.description,
            cat.asset.is_visible
        )
        .fetch_one(&mut *self.tx)
        .await?
        .id;

        query!(
            "
            INSERT INTO tokens (asset_id, ticker)
            VALUES (?, ?)
            ON CONFLICT(asset_id) DO UPDATE SET
                ticker = COALESCE(ticker, excluded.ticker)
            ",
            asset_id,
            cat.ticker,
        )
        .execute(&mut *self.tx)
        .await?;

        Ok(())
    }
}
