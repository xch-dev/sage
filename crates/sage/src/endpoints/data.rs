use base64::{prelude::BASE64_STANDARD, Engine};
use chia::{
    clvm_traits::{FromClvm, ToClvm},
    puzzles::nft::NftMetadata,
};
use chia_wallet_sdk::encode_address;
use clvmr::Allocator;
use hex_literal::hex;
use sage_api::{
    Amount, CatRecord, CoinRecord, DerivationRecord, DidRecord, GetCat, GetCatCoins,
    GetCatCoinsResponse, GetCatResponse, GetCats, GetCatsResponse, GetDerivations,
    GetDerivationsResponse, GetDids, GetDidsResponse, GetNft, GetNftCollection,
    GetNftCollectionResponse, GetNftCollections, GetNftCollectionsResponse, GetNftResponse,
    GetNftStatus, GetNftStatusResponse, GetNfts, GetNftsResponse, GetPendingTransactions,
    GetPendingTransactionsResponse, GetSyncStatus, GetSyncStatusResponse, GetXchCoins,
    GetXchCoinsResponse, NftCollectionRecord, NftInfo, NftRecord, NftSortMode,
    PendingTransactionRecord,
};
use sage_database::{NftData, NftRow};
use sage_wallet::WalletError;

use crate::{parse_asset_id, parse_collection_id, parse_nft_id, Result, Sage};

impl Sage {
    pub async fn get_sync_status(&self, _req: GetSyncStatus) -> Result<GetSyncStatusResponse> {
        let wallet = self.wallet()?;

        let balance = wallet.db.balance().await?;
        let total_coins = wallet.db.total_coin_count().await?;
        let synced_coins = wallet.db.synced_coin_count().await?;

        let puzzle_hash = match wallet.p2_puzzle_hash(false, false).await {
            Ok(puzzle_hash) => Some(puzzle_hash),
            Err(WalletError::InsufficientDerivations) => None,
            Err(error) => return Err(error.into()),
        };

        let receive_address = puzzle_hash
            .map(|puzzle_hash| {
                encode_address(puzzle_hash.to_bytes(), &self.network().address_prefix)
            })
            .transpose()?;

        Ok(GetSyncStatusResponse {
            balance: Amount::u128(balance),
            unit: self.unit.clone(),
            total_coins,
            synced_coins,
            receive_address: receive_address.unwrap_or_default(),
            burn_address: encode_address(
                hex!("000000000000000000000000000000000000000000000000000000000000dead"),
                &self.network().address_prefix,
            )?,
        })
    }

    pub async fn get_derivations(&self, req: GetDerivations) -> Result<GetDerivationsResponse> {
        let wallet = self.wallet()?;

        let derivations = wallet
            .db
            .unhardened_derivations(req.limit, req.offset)
            .await?
            .into_iter()
            .map(|row| {
                Ok(DerivationRecord {
                    index: row.index,
                    public_key: hex::encode(row.synthetic_key.to_bytes()),
                    address: encode_address(
                        row.p2_puzzle_hash.to_bytes(),
                        &self.network().address_prefix,
                    )?,
                })
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(GetDerivationsResponse { derivations })
    }

    pub async fn get_xch_coins(&self, _req: GetXchCoins) -> Result<GetXchCoinsResponse> {
        let wallet = self.wallet()?;

        let mut coins = Vec::new();

        let rows = wallet.db.p2_coin_states().await?;

        for row in rows {
            let cs = row.coin_state;

            let spend_transaction_id = wallet
                .db
                .coin_transaction_id(cs.coin.coin_id())
                .await?
                .map(hex::encode);

            let offer_id = wallet
                .db
                .coin_offer_id(cs.coin.coin_id())
                .await?
                .map(hex::encode);

            coins.push(CoinRecord {
                coin_id: hex::encode(cs.coin.coin_id()),
                address: encode_address(
                    cs.coin.puzzle_hash.to_bytes(),
                    &self.network().address_prefix,
                )?,
                amount: Amount::u64(cs.coin.amount),
                created_height: cs.created_height,
                spent_height: cs.spent_height,
                create_transaction_id: row.transaction_id.map(hex::encode),
                spend_transaction_id,
                offer_id,
            });
        }

        Ok(GetXchCoinsResponse { coins })
    }

    pub async fn get_cat_coins(&self, req: GetCatCoins) -> Result<GetCatCoinsResponse> {
        let wallet = self.wallet()?;
        let asset_id = parse_asset_id(req.asset_id)?;

        let mut coins = Vec::new();

        let rows = wallet.db.cat_coin_states(asset_id).await?;

        for row in rows {
            let cs = row.coin_state;

            let spend_transaction_id = wallet
                .db
                .coin_transaction_id(cs.coin.coin_id())
                .await?
                .map(hex::encode);

            let offer_id = wallet
                .db
                .coin_offer_id(cs.coin.coin_id())
                .await?
                .map(hex::encode);

            coins.push(CoinRecord {
                coin_id: hex::encode(cs.coin.coin_id()),
                address: encode_address(
                    cs.coin.puzzle_hash.to_bytes(),
                    &self.network().address_prefix,
                )?,
                amount: Amount::u64(cs.coin.amount),
                created_height: cs.created_height,
                spent_height: cs.spent_height,
                create_transaction_id: row.transaction_id.map(hex::encode),
                spend_transaction_id,
                offer_id,
            });
        }

        Ok(GetCatCoinsResponse { coins })
    }

    pub async fn get_cats(&self, _req: GetCats) -> Result<GetCatsResponse> {
        let wallet = self.wallet()?;
        let cats = wallet.db.cats_by_name().await?;

        let mut records = Vec::with_capacity(cats.len());

        for cat in cats {
            let balance = wallet.db.cat_balance(cat.asset_id).await?;

            records.push(CatRecord {
                asset_id: hex::encode(cat.asset_id),
                name: cat.name,
                ticker: cat.ticker,
                description: cat.description,
                icon_url: cat.icon,
                visible: cat.visible,
                balance: Amount::u128(balance),
            });
        }

        Ok(GetCatsResponse { cats: records })
    }

    pub async fn get_cat(&self, req: GetCat) -> Result<GetCatResponse> {
        let wallet = self.wallet()?;

        let asset_id = parse_asset_id(req.asset_id)?;
        let cat = wallet.db.cat(asset_id).await?;
        let balance = wallet.db.cat_balance(asset_id).await?;

        let cat = cat
            .map(|cat| {
                Result::Ok(CatRecord {
                    asset_id: hex::encode(cat.asset_id),
                    name: cat.name,
                    ticker: cat.ticker,
                    description: cat.description,
                    icon_url: cat.icon,
                    visible: cat.visible,
                    balance: Amount::u128(balance),
                })
            })
            .transpose()?;

        Ok(GetCatResponse { cat })
    }

    pub async fn get_dids(&self, _req: GetDids) -> Result<GetDidsResponse> {
        let wallet = self.wallet()?;

        let mut dids = Vec::new();

        for row in wallet.db.dids_by_name().await? {
            let Some(did) = wallet.db.did_coin_info(row.coin_id).await? else {
                continue;
            };

            dids.push(DidRecord {
                launcher_id: encode_address(row.launcher_id.to_bytes(), "did:chia:")?,
                name: row.name,
                visible: row.visible,
                coin_id: hex::encode(did.coin_id),
                address: encode_address(
                    did.p2_puzzle_hash.to_bytes(),
                    &self.network().address_prefix,
                )?,
                amount: Amount::u64(did.amount),
                created_height: did.created_height,
                create_transaction_id: did.transaction_id.map(hex::encode),
            });
        }

        Ok(GetDidsResponse { dids })
    }

    pub async fn get_pending_transactions(
        &self,
        _req: GetPendingTransactions,
    ) -> Result<GetPendingTransactionsResponse> {
        let wallet = self.wallet()?;

        let transactions = wallet
            .db
            .transactions()
            .await?
            .into_iter()
            .map(|tx| {
                Result::Ok(PendingTransactionRecord {
                    transaction_id: hex::encode(tx.transaction_id),
                    fee: Amount::u64(tx.fee),
                    // TODO: Date format?
                    submitted_at: tx.submitted_at.map(|ts| ts.to_string()),
                })
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(GetPendingTransactionsResponse { transactions })
    }

    pub async fn get_nft_status(&self, _req: GetNftStatus) -> Result<GetNftStatusResponse> {
        let wallet = self.wallet()?;

        let nfts = wallet.db.nft_count().await?;
        let collections = wallet.db.collection_count().await?;
        let visible_nfts = wallet.db.visible_nft_count().await?;
        let visible_collections = wallet.db.visible_collection_count().await?;

        Ok(GetNftStatusResponse {
            nfts,
            visible_nfts,
            collections,
            visible_collections,
        })
    }

    pub async fn get_nft_collections(
        &self,
        req: GetNftCollections,
    ) -> Result<GetNftCollectionsResponse> {
        let wallet = self.wallet()?;

        let mut records = Vec::new();

        let collections = if req.include_hidden {
            wallet.db.collections_named(req.limit, req.offset).await?
        } else {
            wallet
                .db
                .collections_visible_named(req.limit, req.offset)
                .await?
        };

        for col in collections {
            let total = wallet.db.collection_nft_count(col.collection_id).await?;
            let total_visible = wallet
                .db
                .collection_visible_nft_count(col.collection_id)
                .await?;

            records.push(NftCollectionRecord {
                collection_id: encode_address(col.collection_id.to_bytes(), "col")?,
                did_id: encode_address(col.did_id.to_bytes(), "did:chia:")?,
                metadata_collection_id: col.metadata_collection_id,
                visible: col.visible,
                name: col.name,
                icon: col.icon,
                nfts: total,
                visible_nfts: total_visible,
            });
        }

        Ok(GetNftCollectionsResponse {
            collections: records,
        })
    }

    pub async fn get_nft_collection(
        &self,
        req: GetNftCollection,
    ) -> Result<GetNftCollectionResponse> {
        let wallet = self.wallet()?;

        let collection_id = req.collection_id.map(parse_collection_id).transpose()?;

        let collection = if let Some(collection_id) = collection_id {
            Some(wallet.db.collection(collection_id).await?)
        } else {
            None
        };

        let total = if let Some(collection_id) = collection_id {
            wallet.db.collection_nft_count(collection_id).await?
        } else {
            wallet.db.no_collection_nft_count().await?
        };

        let total_visible = if let Some(collection_id) = collection_id {
            wallet
                .db
                .collection_visible_nft_count(collection_id)
                .await?
        } else {
            wallet.db.no_collection_visible_nft_count().await?
        };

        let record = if let Some(collection) = collection {
            NftCollectionRecord {
                collection_id: encode_address(collection.collection_id.to_bytes(), "col")?,
                did_id: encode_address(collection.did_id.to_bytes(), "did:chia:")?,
                metadata_collection_id: collection.metadata_collection_id,
                visible: collection.visible,
                name: collection.name,
                icon: collection.icon,
                nfts: total,
                visible_nfts: total_visible,
            }
        } else {
            NftCollectionRecord {
                collection_id: "None".to_string(),
                did_id: "Miscellaneous".to_string(),
                metadata_collection_id: "None".to_string(),
                visible: true,
                name: Some("Uncategorized".to_string()),
                icon: None,
                nfts: total,
                visible_nfts: total_visible,
            }
        };

        Ok(GetNftCollectionResponse {
            collection: Some(record),
        })
    }

    pub async fn get_nfts(&self, req: GetNfts) -> Result<GetNftsResponse> {
        let wallet = self.wallet()?;

        let mut records = Vec::new();

        if req.collection_id.as_deref() == Some("all") {
            let nfts = match (req.sort_mode, req.include_hidden) {
                (NftSortMode::Name, true) => wallet.db.nfts_named(req.limit, req.offset).await?,
                (NftSortMode::Name, false) => {
                    wallet.db.nfts_visible_named(req.limit, req.offset).await?
                }
                (NftSortMode::Recent, true) => wallet.db.nfts_recent(req.limit, req.offset).await?,
                (NftSortMode::Recent, false) => {
                    wallet.db.nfts_visible_recent(req.limit, req.offset).await?
                }
            };

            for nft in nfts {
                let data = if let Some(hash) = wallet.db.data_hash(nft.launcher_id).await? {
                    wallet.db.fetch_nft_data(hash).await?
                } else {
                    None
                };

                let collection_name = if let Some(collection_id) = nft.collection_id {
                    wallet.db.collection_name(collection_id).await?
                } else {
                    None
                };

                records.push(nft_record(nft, collection_name, data)?);
            }
        } else {
            let collection_id = req.collection_id.map(parse_collection_id).transpose()?;

            let nfts = match (req.sort_mode, req.include_hidden, collection_id) {
                (NftSortMode::Name, true, Some(collection_id)) => {
                    wallet
                        .db
                        .collection_nfts_named(collection_id, req.limit, req.offset)
                        .await?
                }
                (NftSortMode::Name, false, Some(collection_id)) => {
                    wallet
                        .db
                        .collection_nfts_visible_named(collection_id, req.limit, req.offset)
                        .await?
                }
                (NftSortMode::Recent, true, Some(collection_id)) => {
                    wallet
                        .db
                        .collection_nfts_recent(collection_id, req.limit, req.offset)
                        .await?
                }
                (NftSortMode::Recent, false, Some(collection_id)) => {
                    wallet
                        .db
                        .collection_nfts_visible_recent(collection_id, req.limit, req.offset)
                        .await?
                }
                (NftSortMode::Name, true, None) => {
                    wallet
                        .db
                        .no_collection_nfts_named(req.limit, req.offset)
                        .await?
                }
                (NftSortMode::Name, false, None) => {
                    wallet
                        .db
                        .no_collection_nfts_visible_named(req.limit, req.offset)
                        .await?
                }
                (NftSortMode::Recent, true, None) => {
                    wallet
                        .db
                        .no_collection_nfts_recent(req.limit, req.offset)
                        .await?
                }
                (NftSortMode::Recent, false, None) => {
                    wallet
                        .db
                        .no_collection_nfts_visible_recent(req.limit, req.offset)
                        .await?
                }
            };

            for nft in nfts {
                let data = if let Some(hash) = wallet.db.data_hash(nft.launcher_id).await? {
                    wallet.db.fetch_nft_data(hash).await?
                } else {
                    None
                };

                let collection_name = if let Some(collection_id) = nft.collection_id {
                    wallet.db.collection_name(collection_id).await?
                } else {
                    None
                };

                records.push(nft_record(nft, collection_name, data)?);
            }
        }

        Ok(GetNftsResponse { nfts: records })
    }

    pub async fn get_nft(&self, req: GetNft) -> Result<GetNftResponse> {
        let wallet = self.wallet()?;

        let nft_id = parse_nft_id(req.nft_id)?;

        let Some(nft_row) = wallet.db.nft_row(nft_id).await? else {
            return Ok(GetNftResponse { nft: None });
        };

        let Some(nft) = wallet.db.nft(nft_id).await? else {
            return Ok(GetNftResponse { nft: None });
        };

        let mut allocator = Allocator::new();
        let metadata_ptr = nft.info.metadata.to_clvm(&mut allocator)?;
        let metadata = NftMetadata::from_clvm(&allocator, metadata_ptr).ok();

        let data_hash = metadata.as_ref().and_then(|m| m.data_hash);
        let metadata_hash = metadata.as_ref().and_then(|m| m.metadata_hash);
        let license_hash = metadata.as_ref().and_then(|m| m.license_hash);

        let data = if let Some(hash) = data_hash {
            wallet.db.fetch_nft_data(hash).await?
        } else {
            None
        };

        let offchain_metadata = if let Some(hash) = metadata_hash {
            wallet.db.fetch_nft_data(hash).await?
        } else {
            None
        };

        let collection_name = if let Some(collection_id) = nft_row.collection_id {
            wallet.db.collection_name(collection_id).await?
        } else {
            None
        };

        Ok(GetNftResponse {
            nft: Some(NftInfo {
                launcher_id: encode_address(nft_row.launcher_id.to_bytes(), "nft")?,
                collection_id: nft_row
                    .collection_id
                    .map(|col| encode_address(col.to_bytes(), "col"))
                    .transpose()?,
                collection_name,
                minter_did: nft_row
                    .minter_did
                    .map(|did| encode_address(did.to_bytes(), "did:chia:"))
                    .transpose()?,
                owner_did: nft_row
                    .owner_did
                    .map(|did| encode_address(did.to_bytes(), "did:chia:"))
                    .transpose()?,
                visible: nft_row.visible,
                coin_id: hex::encode(nft.coin.coin_id()),
                address: encode_address(
                    nft.info.p2_puzzle_hash.to_bytes(),
                    &self.network().address_prefix,
                )?,
                royalty_address: encode_address(
                    nft.info.royalty_puzzle_hash.to_bytes(),
                    &self.network().address_prefix,
                )?,
                royalty_ten_thousandths: nft.info.royalty_ten_thousandths,
                data_uris: metadata
                    .as_ref()
                    .map(|m| m.data_uris.clone())
                    .unwrap_or_default(),
                data_hash: data_hash.map(hex::encode),
                metadata_uris: metadata
                    .as_ref()
                    .map(|m| m.metadata_uris.clone())
                    .unwrap_or_default(),
                metadata_hash: metadata_hash.map(hex::encode),
                license_uris: metadata
                    .as_ref()
                    .map(|m| m.license_uris.clone())
                    .unwrap_or_default(),
                license_hash: license_hash.map(hex::encode),
                edition_number: metadata
                    .as_ref()
                    .map(|m| m.edition_number.try_into())
                    .transpose()?,
                edition_total: metadata
                    .as_ref()
                    .map(|m| m.edition_total.try_into())
                    .transpose()?,
                created_height: nft_row.created_height,
                data: data.as_ref().map(|data| BASE64_STANDARD.encode(&data.blob)),
                data_mime_type: data.map(|data| data.mime_type),
                metadata: offchain_metadata.and_then(|offchain_metadata| {
                    if offchain_metadata.mime_type == "application/json" {
                        String::from_utf8(offchain_metadata.blob).ok()
                    } else {
                        None
                    }
                }),
            }),
        })
    }
}

fn nft_record(
    nft: NftRow,
    collection_name: Option<String>,
    data: Option<NftData>,
) -> Result<NftRecord> {
    Ok(NftRecord {
        launcher_id: encode_address(nft.launcher_id.to_bytes(), "nft")?,
        collection_id: nft
            .collection_id
            .map(|col| encode_address(col.to_bytes(), "col"))
            .transpose()?,
        collection_name,
        minter_did: nft
            .minter_did
            .map(|did| encode_address(did.to_bytes(), "did:chia:"))
            .transpose()?,
        owner_did: nft
            .owner_did
            .map(|did| encode_address(did.to_bytes(), "did:chia:"))
            .transpose()?,
        visible: nft.visible,
        sensitive_content: nft.sensitive_content,
        name: nft.name,
        data: data.as_ref().map(|data| BASE64_STANDARD.encode(&data.blob)),
        data_mime_type: data.map(|data| data.mime_type),
        created_height: nft.created_height,
    })
}
