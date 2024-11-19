use sage_api::{
    RemoveCat, RemoveCatResponse, UpdateCat, UpdateCatResponse, UpdateDid, UpdateDidResponse,
    UpdateNft, UpdateNftResponse,
};
use sage_database::{CatRow, DidRow};

use crate::{parse_asset_id, parse_did_id, parse_nft_id, Error, Result, Sage};

impl Sage {
    pub async fn remove_cat(&self, req: RemoveCat) -> Result<RemoveCatResponse> {
        let wallet = self.wallet()?;

        let asset_id = parse_asset_id(req.asset_id)?;
        wallet.db.refetch_cat(asset_id).await?;

        Ok(RemoveCatResponse {})
    }

    pub async fn update_cat(&self, req: UpdateCat) -> Result<UpdateCatResponse> {
        let wallet = self.wallet()?;

        let asset_id = parse_asset_id(req.record.asset_id)?;

        wallet
            .db
            .update_cat(CatRow {
                asset_id,
                name: req.record.name,
                description: req.record.description,
                ticker: req.record.ticker,
                icon: req.record.icon_url,
                visible: req.record.visible,
                fetched: true,
            })
            .await?;

        Ok(UpdateCatResponse {})
    }

    pub async fn update_did(&self, req: UpdateDid) -> Result<UpdateDidResponse> {
        let wallet = self.wallet()?;

        let did_id = parse_did_id(req.did_id)?;

        let Some(row) = wallet.db.did_row(did_id).await? else {
            return Err(Error::MissingDid(did_id));
        };

        wallet
            .db
            .insert_did(DidRow {
                launcher_id: row.launcher_id,
                coin_id: row.coin_id,
                name: req.name,
                is_owned: row.is_owned,
                visible: req.visible,
                created_height: row.created_height,
            })
            .await?;

        Ok(UpdateDidResponse {})
    }

    pub async fn update_nft(&self, req: UpdateNft) -> Result<UpdateNftResponse> {
        let wallet = self.wallet()?;

        let nft_id = parse_nft_id(req.nft_id)?;
        wallet.db.set_nft_visible(nft_id, req.visible).await?;

        Ok(UpdateNftResponse {})
    }
}
