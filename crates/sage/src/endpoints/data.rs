use base64::{prelude::BASE64_STANDARD, Engine};
use chia::{
    clvm_traits::{FromClvm, ToClvm},
    protocol::{Bytes32, Program},
    puzzles::nft::NftMetadata,
};
use chia_puzzles::{SETTLEMENT_PAYMENT_HASH, SINGLETON_LAUNCHER_HASH};
use chia_wallet_sdk::{driver::Nft, utils::Address};
use clvmr::Allocator;
use sage_api::{
    AddressKind, Amount, AssetKind, CatRecord, CheckAddress, CheckAddressResponse, CoinRecord,
    DerivationRecord, DidRecord, GetCat, GetCatCoins, GetCatCoinsResponse, GetCatResponse, GetCats,
    GetCatsResponse, GetDerivations, GetDerivationsResponse, GetDids, GetDidsResponse,
    GetMinterDidIds, GetMinterDidIdsResponse, GetNft, GetNftCollection, GetNftCollectionResponse,
    GetNftCollections, GetNftCollectionsResponse, GetNftData, GetNftDataResponse, GetNftIcon,
    GetNftIconResponse, GetNftResponse, GetNftThumbnail, GetNftThumbnailResponse, GetNfts,
    GetNftsResponse, GetPendingTransactions, GetPendingTransactionsResponse, GetSyncStatus,
    GetSyncStatusResponse, GetTransaction, GetTransactionResponse, GetTransactions,
    GetTransactionsByItemId, GetTransactionsByItemIdResponse, GetTransactionsResponse, GetXchCoins,
    GetXchCoinsResponse, NftCollectionRecord, NftData, NftRecord, NftSortMode as ApiNftSortMode,
    PendingTransactionRecord, TransactionCoin, TransactionRecord,
};
use sage_database::{
    CoinKind, CoinStateRow, Database, NftGroup, NftRow, NftSearchParams, NftSortMode,
};
use sage_wallet::WalletError;

use crate::{
    parse_asset_id, parse_collection_id, parse_did_id, parse_nft_id, Error, Result, Sage,
    BURN_PUZZLE_HASH,
};

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
            .map(|puzzle_hash| Address::new(puzzle_hash, self.network().prefix()).encode())
            .transpose()?;

        Ok(GetSyncStatusResponse {
            balance: Amount::u128(balance),
            unit: self.unit.clone(),
            total_coins,
            synced_coins,
            receive_address: receive_address.unwrap_or_default(),
            burn_address: Address::new(BURN_PUZZLE_HASH.into(), self.network().prefix())
                .encode()?,
            unhardened_derivation_index: wallet.db.derivation_index(false).await?,
            hardened_derivation_index: wallet.db.derivation_index(true).await?,
        })
    }

    pub async fn check_address(&self, req: CheckAddress) -> Result<CheckAddressResponse> {
        let wallet = self.wallet()?;

        let Some(address) = Address::decode(&req.address).ok() else {
            return Ok(CheckAddressResponse { valid: false });
        };

        let is_valid = wallet.db.is_p2_puzzle_hash(address.puzzle_hash).await?;

        Ok(CheckAddressResponse { valid: is_valid })
    }

    pub async fn get_derivations(&self, req: GetDerivations) -> Result<GetDerivationsResponse> {
        let wallet = self.wallet()?;

        let derivations = wallet
            .db
            .derivations(req.hardened, req.limit, req.offset)
            .await?
            .into_iter()
            .map(|row| {
                Ok(DerivationRecord {
                    index: row.index,
                    public_key: hex::encode(row.synthetic_key.to_bytes()),
                    address: Address::new(row.p2_puzzle_hash, self.network().prefix()).encode()?,
                })
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(GetDerivationsResponse { derivations })
    }

    pub async fn get_xch_coins(&self, _req: GetXchCoins) -> Result<GetXchCoinsResponse> {
        let wallet = self.wallet()?;

        let mut coins = Vec::new();
        let (rows, total) = wallet.db.p2_coin_states(_req.limit, _req.offset).await?;

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
                address: Address::new(cs.coin.puzzle_hash, self.network().prefix()).encode()?,
                amount: Amount::u64(cs.coin.amount),
                created_height: cs.created_height,
                spent_height: cs.spent_height,
                create_transaction_id: row.transaction_id.map(hex::encode),
                spend_transaction_id,
                offer_id,
                created_timestamp: row.created_timestamp,
                spent_timestamp: row.spent_timestamp,
            });
        }

        Ok(GetXchCoinsResponse { coins, total })
    }

    pub async fn get_cat_coins(&self, req: GetCatCoins) -> Result<GetCatCoinsResponse> {
        let wallet = self.wallet()?;
        let asset_id = parse_asset_id(req.asset_id)?;

        let mut coins = Vec::new();

        let (rows, total) = wallet
            .db
            .cat_coin_states(asset_id, req.limit, req.offset)
            .await?;

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
                address: Address::new(cs.coin.puzzle_hash, self.network().prefix()).encode()?,
                amount: Amount::u64(cs.coin.amount),
                created_height: cs.created_height,
                spent_height: cs.spent_height,
                create_transaction_id: row.transaction_id.map(hex::encode),
                spend_transaction_id,
                offer_id,
                created_timestamp: row.created_timestamp,
                spent_timestamp: row.spent_timestamp,
            });
        }

        Ok(GetCatCoinsResponse { coins, total })
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
                launcher_id: Address::new(row.launcher_id, "did:chia:".to_string()).encode()?,
                name: row.name,
                visible: row.visible,
                coin_id: hex::encode(did.coin_id),
                address: Address::new(did.p2_puzzle_hash, self.network().prefix()).encode()?,
                amount: Amount::u64(did.amount),
                recovery_hash: did.recovery_list_hash.map(hex::encode),
                created_height: did.created_height,
                create_transaction_id: did.transaction_id.map(hex::encode),
            });
        }

        Ok(GetDidsResponse { dids })
    }

    pub async fn get_minter_did_ids(
        &self,
        req: GetMinterDidIds,
    ) -> Result<GetMinterDidIdsResponse> {
        let wallet = self.wallet()?;

        let (dids, total) = wallet
            .db
            .distinct_minter_dids(req.limit, req.offset)
            .await?;

        let did_ids = dids
            .into_iter()
            .filter_map(|did| did.map(|d| Address::new(d, "did:chia:".to_string()).encode().ok()))
            .flatten()
            .collect();

        Ok(GetMinterDidIdsResponse { did_ids, total })
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

    pub async fn get_transactions(&self, req: GetTransactions) -> Result<GetTransactionsResponse> {
        let wallet = self.wallet()?;

        let mut transactions = Vec::new();

        let (heights, total) = wallet
            .db
            .get_block_heights(req.offset, req.limit, req.ascending, req.find_value)
            .await?;
        for height in heights {
            let transaction = self.transaction_record(&wallet.db, height).await?;
            transactions.push(transaction);
        }

        // Note: The actual summarization logic will be implemented later
        // For now, we're just passing the parameter through

        Ok(GetTransactionsResponse {
            transactions,
            total,
        })
    }

    pub async fn get_transactions_by_item_id(
        &self,
        req: GetTransactionsByItemId,
    ) -> Result<GetTransactionsByItemIdResponse> {
        let wallet = self.wallet()?;

        let mut transactions = Vec::new();

        let (heights, total) = wallet
            .db
            .get_block_heights_by_item_id(req.offset, req.limit, req.ascending, req.id)
            .await?;
        for height in heights {
            let transaction = self.transaction_record(&wallet.db, height).await?;
            transactions.push(transaction);
        }

        // Note: The actual summarization logic will be implemented later
        // For now, we're just passing the parameter through

        Ok(GetTransactionsByItemIdResponse {
            transactions,
            total,
        })
    }

    pub async fn get_transaction(&self, req: GetTransaction) -> Result<GetTransactionResponse> {
        let wallet = self.wallet()?;
        let transaction = self.transaction_record(&wallet.db, req.height).await?;
        Ok(GetTransactionResponse { transaction })
    }

    pub async fn get_nft_collections(
        &self,
        req: GetNftCollections,
    ) -> Result<GetNftCollectionsResponse> {
        let wallet = self.wallet()?;
        let include_hidden = req.include_hidden;

        let (collections, total) = if include_hidden {
            wallet.db.collections_named(req.limit, req.offset).await?
        } else {
            wallet
                .db
                .collections_visible_named(req.limit, req.offset)
                .await?
        };

        let records = collections
            .into_iter()
            .map(|row| {
                Ok(NftCollectionRecord {
                    collection_id: Address::new(row.collection_id, "col".to_string()).encode()?,
                    did_id: Address::new(row.did_id, "did:chia:".to_string()).encode()?,
                    metadata_collection_id: row.metadata_collection_id,
                    name: row.name,
                    icon: row.icon,
                    visible: row.visible,
                })
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(GetNftCollectionsResponse {
            collections: records,
            total,
        })
    }

    pub async fn get_nft_collection(
        &self,
        req: GetNftCollection,
    ) -> Result<GetNftCollectionResponse> {
        let wallet = self.wallet()?;

        let collection_id = req.collection_id.map(parse_collection_id).transpose()?;

        let collection = if let Some(collection_id) = collection_id {
            let Some(collection) = wallet.db.collection(collection_id).await? else {
                return Ok(GetNftCollectionResponse { collection: None });
            };
            Some(collection)
        } else {
            None
        };

        let record = if let Some(collection) = collection {
            NftCollectionRecord {
                collection_id: Address::new(collection.collection_id, "col".to_string())
                    .encode()?,
                did_id: Address::new(collection.did_id, "did:chia:".to_string()).encode()?,
                metadata_collection_id: collection.metadata_collection_id,
                visible: collection.visible,
                name: collection.name,
                icon: collection.icon,
            }
        } else {
            NftCollectionRecord {
                collection_id: "None".to_string(),
                did_id: "Miscellaneous".to_string(),
                metadata_collection_id: "None".to_string(),
                visible: true,
                name: Some("Uncategorized".to_string()),
                icon: None,
            }
        };

        Ok(GetNftCollectionResponse {
            collection: Some(record),
        })
    }

    pub async fn get_nfts(&self, req: GetNfts) -> Result<GetNftsResponse> {
        let wallet = self.wallet()?;

        let mut records = Vec::new();

        let group = match (&req.collection_id, &req.minter_did_id, &req.owner_did_id) {
            (Some(collection_id), None, None) => {
                if collection_id == "none" {
                    Some(NftGroup::NoCollection)
                } else {
                    Some(NftGroup::Collection(parse_collection_id(
                        collection_id.clone(),
                    )?))
                }
            }
            (None, Some(minter_did_id), None) => {
                if minter_did_id == "none" {
                    Some(NftGroup::NoMinterDid)
                } else {
                    Some(NftGroup::MinterDid(parse_did_id(minter_did_id.clone())?))
                }
            }
            (None, None, Some(owner_did_id)) => {
                if owner_did_id == "none" {
                    Some(NftGroup::NoOwnerDid)
                } else {
                    Some(NftGroup::OwnerDid(parse_did_id(owner_did_id.clone())?))
                }
            }
            (None, None, None) => None,
            _ => return Err(Error::InvalidGroup),
        };

        let params = NftSearchParams {
            sort_mode: match req.sort_mode {
                ApiNftSortMode::Recent => NftSortMode::Recent,
                ApiNftSortMode::Name => NftSortMode::Name,
            },
            include_hidden: req.include_hidden,
            group,
            name: req.name,
        };

        let (nfts, total) = wallet.db.search_nfts(params, req.limit, req.offset).await?;

        for nft_row in nfts {
            let Some(nft) = wallet.db.nft(nft_row.launcher_id).await? else {
                continue;
            };

            let collection_name = if let Some(collection_id) = nft_row.collection_id {
                wallet.db.collection_name(collection_id).await?
            } else {
                None
            };

            records.push(self.nft_record(nft_row, nft, collection_name)?);
        }

        Ok(GetNftsResponse {
            nfts: records,
            total,
        })
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

        let collection_name = if let Some(collection_id) = nft_row.collection_id {
            wallet.db.collection_name(collection_id).await?
        } else {
            None
        };

        Ok(GetNftResponse {
            nft: Some(self.nft_record(nft_row, nft, collection_name)?),
        })
    }

    pub async fn get_nft_data(&self, req: GetNftData) -> Result<GetNftDataResponse> {
        let wallet = self.wallet()?;

        let nft_id = parse_nft_id(req.nft_id)?;

        let Some(nft) = wallet.db.nft(nft_id).await? else {
            return Ok(GetNftDataResponse { data: None });
        };

        let mut allocator = Allocator::new();
        let metadata_ptr = nft.info.metadata.to_clvm(&mut allocator)?;
        let metadata = NftMetadata::from_clvm(&allocator, metadata_ptr).ok();

        let data_hash = metadata.as_ref().and_then(|m| m.data_hash);
        let metadata_hash = metadata.as_ref().and_then(|m| m.metadata_hash);

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

        let hash_matches = data.as_ref().is_some_and(|data| data.hash_matches);
        let metadata_hash_matches = offchain_metadata
            .as_ref()
            .is_some_and(|offchain_metadata| offchain_metadata.hash_matches);

        Ok(GetNftDataResponse {
            data: Some(NftData {
                blob: data.as_ref().map(|data| BASE64_STANDARD.encode(&data.blob)),
                mime_type: data.map(|data| data.mime_type),
                hash_matches,
                metadata_json: offchain_metadata.and_then(|offchain_metadata| {
                    if offchain_metadata.mime_type == "application/json" {
                        String::from_utf8(offchain_metadata.blob).ok()
                    } else {
                        None
                    }
                }),
                metadata_hash_matches,
            }),
        })
    }

    pub async fn get_nft_icon(&self, req: GetNftIcon) -> Result<GetNftIconResponse> {
        let wallet = self.wallet()?;

        let nft_id = parse_nft_id(req.nft_id)?;

        let Some(nft) = wallet.db.nft(nft_id).await? else {
            return Ok(GetNftIconResponse { icon: None });
        };

        let mut allocator = Allocator::new();
        let metadata_ptr = nft.info.metadata.to_clvm(&mut allocator)?;
        let metadata = NftMetadata::from_clvm(&allocator, metadata_ptr).ok();

        let Some(data_hash) = metadata.as_ref().and_then(|m| m.data_hash) else {
            return Ok(GetNftIconResponse { icon: None });
        };

        Ok(GetNftIconResponse {
            icon: wallet
                .db
                .nft_icon(data_hash)
                .await?
                .map(|icon| BASE64_STANDARD.encode(icon)),
        })
    }

    pub async fn get_nft_thumbnail(&self, req: GetNftThumbnail) -> Result<GetNftThumbnailResponse> {
        let wallet = self.wallet()?;

        let nft_id = parse_nft_id(req.nft_id)?;

        let Some(nft) = wallet.db.nft(nft_id).await? else {
            return Ok(GetNftThumbnailResponse { thumbnail: None });
        };

        let mut allocator = Allocator::new();
        let metadata_ptr = nft.info.metadata.to_clvm(&mut allocator)?;
        let metadata = NftMetadata::from_clvm(&allocator, metadata_ptr).ok();

        let Some(data_hash) = metadata.as_ref().and_then(|m| m.data_hash) else {
            return Ok(GetNftThumbnailResponse { thumbnail: None });
        };

        Ok(GetNftThumbnailResponse {
            thumbnail: wallet
                .db
                .nft_thumbnail(data_hash)
                .await?
                .map(|thumbnail| BASE64_STANDARD.encode(thumbnail)),
        })
    }

    fn nft_record(
        &self,
        nft_row: NftRow,
        nft: Nft<Program>,
        collection_name: Option<String>,
    ) -> Result<NftRecord> {
        let mut allocator = Allocator::new();
        let metadata_ptr = nft.info.metadata.to_clvm(&mut allocator)?;
        let metadata = NftMetadata::from_clvm(&allocator, metadata_ptr).ok();

        let data_hash = metadata.as_ref().and_then(|m| m.data_hash);
        let metadata_hash = metadata.as_ref().and_then(|m| m.metadata_hash);
        let license_hash = metadata.as_ref().and_then(|m| m.license_hash);

        Ok(NftRecord {
            launcher_id: Address::new(nft_row.launcher_id, "nft".to_string()).encode()?,
            collection_id: nft_row
                .collection_id
                .map(|col| Address::new(col, "col".to_string()).encode())
                .transpose()?,
            collection_name,
            minter_did: nft_row
                .minter_did
                .map(|did| Address::new(did, "did:chia:".to_string()).encode())
                .transpose()?,
            owner_did: nft_row
                .owner_did
                .map(|did| Address::new(did, "did:chia:".to_string()).encode())
                .transpose()?,
            visible: nft_row.visible,
            name: nft_row.name,
            sensitive_content: nft_row.sensitive_content,
            coin_id: hex::encode(nft.coin.coin_id()),
            address: Address::new(nft.info.p2_puzzle_hash, self.network().prefix()).encode()?,
            royalty_address: Address::new(nft.info.royalty_puzzle_hash, self.network().prefix())
                .encode()?,
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
        })
    }

    async fn transaction_coin(&self, db: &Database, coin: CoinStateRow) -> Result<TransactionCoin> {
        let coin_id = coin.coin_state.coin.coin_id();

        let (kind, p2_puzzle_hash) = match coin.kind {
            CoinKind::Unknown => (AssetKind::Unknown, None),
            CoinKind::Xch => (AssetKind::Xch, Some(coin.coin_state.coin.puzzle_hash)),
            CoinKind::Cat => {
                if let Some(cat) = db.cat_coin(coin_id).await? {
                    if let Some(row) = db.cat(cat.asset_id).await? {
                        (
                            AssetKind::Cat {
                                asset_id: hex::encode(cat.asset_id),
                                name: row.name,
                                ticker: row.ticker,
                                icon_url: row.icon,
                            },
                            Some(cat.p2_puzzle_hash),
                        )
                    } else {
                        (
                            AssetKind::Cat {
                                asset_id: hex::encode(cat.asset_id),
                                name: None,
                                ticker: None,
                                icon_url: None,
                            },
                            Some(cat.p2_puzzle_hash),
                        )
                    }
                } else {
                    (AssetKind::Unknown, None)
                }
            }
            CoinKind::Nft => {
                if let Some(nft) = db.nft_by_coin_id(coin_id).await? {
                    let row = db.nft_row(nft.info.launcher_id).await?;

                    let mut allocator = Allocator::new();
                    let metadata_ptr = nft.info.metadata.to_clvm(&mut allocator)?;
                    let metadata = NftMetadata::from_clvm(&allocator, metadata_ptr).ok();

                    let data_hash = metadata.as_ref().and_then(|m| m.data_hash);

                    let icon = if let Some(hash) = data_hash {
                        db.nft_icon(hash).await?
                    } else {
                        None
                    };

                    (
                        AssetKind::Nft {
                            launcher_id: Address::new(nft.info.launcher_id, "nft".to_string())
                                .encode()?,
                            name: row.as_ref().and_then(|row| row.name.clone()),
                            icon: icon.map(|icon| BASE64_STANDARD.encode(icon)),
                        },
                        Some(nft.info.p2_puzzle_hash),
                    )
                } else {
                    (AssetKind::Unknown, None)
                }
            }
            CoinKind::Did => {
                if let Some(did) = db.did_by_coin_id(coin_id).await? {
                    let row = db.did_row(did.info.launcher_id).await?;
                    (
                        AssetKind::Did {
                            launcher_id: Address::new(
                                did.info.launcher_id,
                                "did:chia:".to_string(),
                            )
                            .encode()?,
                            name: row.and_then(|row| row.name),
                        },
                        Some(did.info.p2_puzzle_hash),
                    )
                } else {
                    (AssetKind::Unknown, None)
                }
            }
        };

        let address_kind = if let Some(p2_puzzle_hash) = p2_puzzle_hash {
            self.address_kind(db, p2_puzzle_hash).await?
        } else {
            AddressKind::Unknown
        };

        Ok(TransactionCoin {
            coin_id: hex::encode(coin_id),
            address: p2_puzzle_hash
                .map(|p2_puzzle_hash| {
                    Address::new(p2_puzzle_hash, self.network().prefix()).encode()
                })
                .transpose()?,
            address_kind,
            amount: Amount::u64(coin.coin_state.coin.amount),
            kind,
        })
    }

    async fn transaction_record(&self, db: &Database, height: u32) -> Result<TransactionRecord> {
        let spent_rows = db.get_coin_states_by_spent_height(height).await?;
        let created_rows = db.get_coin_states_by_created_height(height).await?;
        let timestamp = db.check_blockinfo(height).await?;

        let mut spent = Vec::new();
        let mut created = Vec::new();

        for row in spent_rows {
            spent.push(self.transaction_coin(db, row).await?);
        }

        for row in created_rows {
            created.push(self.transaction_coin(db, row).await?);
        }

        Ok(TransactionRecord {
            height,
            timestamp: timestamp.map(TryInto::try_into).transpose()?,
            spent,
            created,
        })
    }

    async fn address_kind(&self, db: &Database, p2_puzzle_hash: Bytes32) -> Result<AddressKind> {
        if p2_puzzle_hash == BURN_PUZZLE_HASH.into() {
            return Ok(AddressKind::Burn);
        } else if p2_puzzle_hash == SINGLETON_LAUNCHER_HASH.into() {
            return Ok(AddressKind::Launcher);
        } else if p2_puzzle_hash == SETTLEMENT_PAYMENT_HASH.into() {
            return Ok(AddressKind::Offer);
        }

        Ok(if db.is_p2_puzzle_hash(p2_puzzle_hash).await? {
            AddressKind::Own
        } else {
            AddressKind::External
        })
    }
}
