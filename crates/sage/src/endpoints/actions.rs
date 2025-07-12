use chia::{
    bls::master_to_wallet_hardened_intermediate,
    clvm_traits::{FromClvm, ToClvm},
    puzzles::{nft::NftMetadata, standard::StandardArgs, DeriveSynthetic},
};
use chia_wallet_sdk::types::TESTNET11_CONSTANTS;
use clvmr::Allocator;
use sage_api::{
    IncreaseDerivationIndex, IncreaseDerivationIndexResponse, RedownloadNft, RedownloadNftResponse,
    ResyncCat, ResyncCatResponse, UpdateCat, UpdateCatResponse, UpdateDid, UpdateDidResponse,
    UpdateNft, UpdateNftCollection, UpdateNftCollectionResponse, UpdateNftResponse,
};
use sage_assets::DexieCat;
use sage_database::{Asset, AssetKind, Derivation};
use sage_wallet::SyncCommand;

use crate::{parse_asset_id, parse_collection_id, parse_did_id, parse_nft_id, Error, Result, Sage};

impl Sage {
    pub async fn resync_cat(&self, req: ResyncCat) -> Result<ResyncCatResponse> {
        let wallet = self.wallet()?;

        let asset_id = parse_asset_id(req.asset_id)?;
        let testnet = self.network().genesis_challenge == TESTNET11_CONSTANTS.genesis_challenge;

        let cat = DexieCat::fetch(asset_id, testnet).await?;

        wallet
            .db
            .update_asset(Asset {
                hash: asset_id,
                name: cat.name,
                ticker: cat.ticker,
                precision: 3,
                icon_url: cat.icon_url,
                description: cat.description,
                is_sensitive_content: false,
                is_visible: true,
                kind: AssetKind::Token,
            })
            .await?;

        Ok(ResyncCatResponse {})
    }

    pub async fn update_cat(&self, req: UpdateCat) -> Result<UpdateCatResponse> {
        let wallet = self.wallet()?;

        let asset_id = parse_asset_id(req.record.asset_id)?;

        let Some(mut asset) = wallet.db.asset(asset_id).await? else {
            return Err(Error::MissingCat(asset_id));
        };

        asset.name = req.record.name;
        asset.ticker = req.record.ticker;
        asset.icon_url = req.record.icon_url;
        asset.description = req.record.description;
        asset.is_visible = req.record.visible;

        wallet.db.update_asset(asset).await?;

        Ok(UpdateCatResponse {})
    }

    pub async fn update_did(&self, req: UpdateDid) -> Result<UpdateDidResponse> {
        let wallet = self.wallet()?;

        let did_id = parse_did_id(req.did_id)?;

        let Some(mut asset) = wallet.db.asset(did_id).await? else {
            return Err(Error::MissingDid(did_id));
        };

        asset.name = req.name;
        asset.is_visible = req.visible;

        wallet.db.update_asset(asset).await?;

        Ok(UpdateDidResponse {})
    }

    pub async fn update_nft(&self, req: UpdateNft) -> Result<UpdateNftResponse> {
        let wallet = self.wallet()?;

        let nft_id = parse_nft_id(req.nft_id)?;

        let Some(mut asset) = wallet.db.asset(nft_id).await? else {
            return Err(Error::MissingNft(nft_id));
        };

        asset.is_visible = req.visible;

        wallet.db.update_asset(asset).await?;

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
                tx.set_uri_unchecked(uri).await?;
            }

            if let Some(hash) = metadata.data_hash {
                tx.delete_file(hash).await?;
            }

            if let Some(hash) = metadata.metadata_hash {
                tx.delete_file(hash).await?;
            }

            if let Some(hash) = metadata.license_hash {
                tx.delete_file(hash).await?;
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

                tx.insert_custody_p2_puzzle(
                    p2_puzzle_hash,
                    synthetic_key,
                    Derivation {
                        derivation_index: index,
                        is_hardened: true,
                    },
                )
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
