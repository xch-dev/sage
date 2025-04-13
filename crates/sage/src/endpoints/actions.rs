use chia::{
    bls::master_to_wallet_hardened_intermediate,
    clvm_traits::{FromClvm, ToClvm},
    puzzles::{nft::NftMetadata, standard::StandardArgs, DeriveSynthetic},
};
use clvmr::Allocator;
use sage_api::{
    IncreaseDerivationIndex, IncreaseDerivationIndexResponse, RedownloadNft, RedownloadNftResponse,
    RemoveCat, RemoveCatResponse, UpdateCat, UpdateCatResponse, UpdateDid, UpdateDidResponse,
    UpdateNft, UpdateNftCollection, UpdateNftCollectionResponse, UpdateNftResponse,
};
use sage_database::{CatRow, DidRow};
use sage_wallet::SyncCommand;

use crate::{parse_asset_id, parse_collection_id, parse_did_id, parse_nft_id, Error, Result, Sage};

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

    pub async fn update_nft_collection(
        &self,
        req: UpdateNftCollection,
    ) -> Result<UpdateNftCollectionResponse> {
        let wallet = self.wallet()?;

        let collection_id = parse_collection_id(req.collection_id)?;
        wallet
            .db
            .set_collection_visible(collection_id, req.visible)
            .await?;

        Ok(UpdateNftCollectionResponse {})
    }

    pub async fn redownload_nft(&self, req: RedownloadNft) -> Result<RedownloadNftResponse> {
        let wallet = self.wallet()?;

        let nft_id = parse_nft_id(req.nft_id)?;

        let Some(nft) = wallet.db.nft(nft_id).await? else {
            return Err(Error::MissingNft(nft_id));
        };

        let mut allocator = Allocator::new();
        let metadata = nft
            .info
            .metadata
            .to_clvm(&mut allocator)
            .ok()
            .and_then(|ptr| NftMetadata::from_clvm(&allocator, ptr).ok());

        if let Some(metadata) = metadata {
            let mut tx = wallet.db.tx().await?;

            for uri in [
                metadata.data_uris,
                metadata.metadata_uris,
                metadata.license_uris,
            ]
            .concat()
            {
                tx.set_nft_uri_unchecked(uri).await?;
            }

            if let Some(hash) = metadata.data_hash {
                tx.delete_nft_data(hash).await?;
                tx.delete_nft_thumbnail(hash).await?;
            }

            if let Some(hash) = metadata.metadata_hash {
                tx.delete_nft_data(hash).await?;
            }

            if let Some(hash) = metadata.license_hash {
                tx.delete_nft_data(hash).await?;
            }

            tx.commit().await?;
        }

        Ok(RedownloadNftResponse {})
    }

    pub async fn increase_derivation_index(
        &self,
        req: IncreaseDerivationIndex,
    ) -> Result<IncreaseDerivationIndexResponse> {
        let wallet = self.wallet()?;

        let hardened = req.hardened.is_none_or(|hardened| hardened);
        let unhardened = req.hardened.is_none_or(|hardened| !hardened);

        let mut derivations = Vec::new();

        if hardened {
            let (_mnemonic, Some(master_sk)) =
                self.keychain.extract_secrets(wallet.fingerprint, b"")?
            else {
                return Err(Error::NoSigningKey);
            };

            let mut tx = wallet.db.tx().await?;

            let start = tx.derivation_index(true).await?;
            let intermediate_sk = master_to_wallet_hardened_intermediate(&master_sk);

            for index in start..req.index {
                let synthetic_key = intermediate_sk
                    .derive_hardened(index)
                    .derive_synthetic()
                    .public_key();

                let p2_puzzle_hash = StandardArgs::curry_tree_hash(synthetic_key).into();

                tx.insert_derivation(p2_puzzle_hash, index, true, synthetic_key)
                    .await?;

                derivations.push(p2_puzzle_hash);
            }

            tx.commit().await?;
        }

        if unhardened {
            let mut tx = wallet.db.tx().await?;

            let start = tx.derivation_index(false).await?;

            derivations.extend(
                wallet
                    .insert_unhardened_derivations(&mut tx, start..req.index)
                    .await?,
            );

            tx.commit().await?;
        }

        self.command_sender
            .send(SyncCommand::SubscribePuzzles {
                puzzle_hashes: derivations,
            })
            .await?;

        Ok(IncreaseDerivationIndexResponse {})
    }
}
