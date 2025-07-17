use crate::{
    address_kind, encode_asset, parse_asset_id, parse_collection_id, parse_did_id, parse_nft_id,
    Error, Result, Sage,
};
use base64::{prelude::BASE64_STANDARD, Engine};
use chia::protocol::Bytes32;
use chia::{
    clvm_traits::{FromClvm, ToClvm},
    puzzles::nft::NftMetadata,
};
use chia_wallet_sdk::{driver::BURN_PUZZLE_HASH, utils::Address};
use clvmr::Allocator;
use sage_api::{
    Amount, CheckAddress, CheckAddressResponse, CoinFilterMode as ApiCoinFilterMode, CoinRecord,
    CoinSortMode as ApiCoinSortMode, DerivationRecord, DidRecord, GetAllCats, GetAllCatsResponse,
    GetAreCoinsSpendable, GetAreCoinsSpendableResponse, GetCat, GetCatResponse, GetCats,
    GetCatsResponse, GetCoins, GetCoinsByIds, GetCoinsByIdsResponse, GetCoinsResponse,
    GetDerivations, GetDerivationsResponse, GetDids, GetDidsResponse, GetMinterDidIds,
    GetMinterDidIdsResponse, GetNft, GetNftCollection, GetNftCollectionResponse, GetNftCollections,
    GetNftCollectionsResponse, GetNftData, GetNftDataResponse, GetNftIcon, GetNftIconResponse,
    GetNftResponse, GetNftThumbnail, GetNftThumbnailResponse, GetNfts, GetNftsResponse,
    GetPendingTransactions, GetPendingTransactionsResponse, GetSpendableCoinCount,
    GetSpendableCoinCountResponse, GetSyncStatus, GetSyncStatusResponse, GetTransaction,
    GetTransactionResponse, GetTransactions, GetTransactionsResponse, GetVersion,
    GetVersionResponse, GetXchToken, GetXchTokenResponse, NftCollectionRecord, NftData, NftRecord,
    NftSortMode as ApiNftSortMode, PendingTransactionRecord, TokenRecord, TransactionCoinRecord,
    TransactionRecord,
};
use sage_database::{
    AssetFilter, CoinFilterMode, CoinSortMode, NftGroupSearch, NftRow, NftSortMode, Transaction,
    TransactionCoin,
};
use sage_wallet::WalletError;

impl Sage {
    pub fn get_version(&self, _req: GetVersion) -> Result<GetVersionResponse> {
        Ok(GetVersionResponse {
            version: env!("CARGO_PKG_VERSION").to_string(),
        })
    }

    pub async fn get_sync_status(&self, _req: GetSyncStatus) -> Result<GetSyncStatusResponse> {
        let wallet = self.wallet()?;

        let balance = wallet.db.xch_balance().await?;
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

        let database_size = self
            .wallet_db_path(wallet.fingerprint)
            .ok()
            .and_then(|path| path.metadata().ok())
            .map_or(0, |metadata| metadata.len());

        Ok(GetSyncStatusResponse {
            balance: Amount::u128(balance),
            unit: self.unit.clone(),
            total_coins,
            synced_coins,
            receive_address: receive_address.unwrap_or_default(),
            burn_address: Address::new(BURN_PUZZLE_HASH, self.network().prefix()).encode()?,
            unhardened_derivation_index: wallet.db.max_derivation_index(false).await?,
            hardened_derivation_index: wallet.db.max_derivation_index(true).await?,
            checked_files: wallet.db.checked_files().await?.try_into().unwrap_or(0),
            total_files: wallet.db.total_files().await?.try_into().unwrap_or(0),
            database_size,
        })
    }

    pub async fn check_address(&self, req: CheckAddress) -> Result<CheckAddressResponse> {
        let wallet = self.wallet()?;

        let Some(address) = Address::decode(&req.address).ok() else {
            return Ok(CheckAddressResponse { valid: false });
        };

        let is_valid = wallet
            .db
            .is_custody_p2_puzzle_hash(address.puzzle_hash)
            .await?;

        Ok(CheckAddressResponse { valid: is_valid })
    }

    pub async fn get_derivations(&self, req: GetDerivations) -> Result<GetDerivationsResponse> {
        let wallet = self.wallet()?;

        let (derivations, total) = wallet
            .db
            .derivations(req.hardened, req.limit, req.offset)
            .await?;

        let derivations = derivations
            .into_iter()
            .map(|row| {
                Ok(DerivationRecord {
                    index: row.index,
                    public_key: hex::encode(row.synthetic_key.to_bytes()),
                    address: Address::new(row.p2_puzzle_hash, self.network().prefix()).encode()?,
                })
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(GetDerivationsResponse { derivations, total })
    }

    pub async fn get_are_coins_spendable(
        &self,
        req: GetAreCoinsSpendable,
    ) -> Result<GetAreCoinsSpendableResponse> {
        let wallet = self.wallet()?;
        let spendable = wallet.db.are_coins_spendable(&req.coin_ids).await?;

        Ok(GetAreCoinsSpendableResponse { spendable })
    }

    pub async fn get_spendable_coin_count(
        &self,
        req: GetSpendableCoinCount,
    ) -> Result<GetSpendableCoinCountResponse> {
        let wallet = self.wallet()?;
        let count = if req.asset_id == "xch" {
            wallet.db.spendable_xch_coin_count().await?
        } else {
            let asset_id = parse_asset_id(req.asset_id)?;

            wallet.db.spendable_cat_coin_count(asset_id).await?
        };

        Ok(GetSpendableCoinCountResponse { count })
    }

    pub async fn get_coins_by_ids(&self, req: GetCoinsByIds) -> Result<GetCoinsByIdsResponse> {
        let wallet = self.wallet()?;
        let rows = wallet.db.coins_by_ids(&req.coin_ids).await?;
        let mut coins = Vec::new();

        for row in rows {
            coins.push(CoinRecord {
                coin_id: hex::encode(row.coin.coin_id()),
                address: Address::new(row.coin.puzzle_hash, self.network().prefix()).encode()?,
                amount: Amount::u64(row.coin.amount),
                created_height: row.created_height,
                spent_height: row.spent_height,
                transaction_id: row.mempool_item_hash.map(hex::encode),
                offer_id: row.offer_hash.map(hex::encode),
                created_timestamp: row.created_timestamp,
                spent_timestamp: row.spent_timestamp,
            });
        }
        Ok(GetCoinsByIdsResponse { coins })
    }

    pub async fn get_coins(&self, req: GetCoins) -> Result<GetCoinsResponse> {
        let wallet = self.wallet()?;
        let sort_mode = match req.sort_mode {
            ApiCoinSortMode::CoinId => CoinSortMode::CoinId,
            ApiCoinSortMode::Amount => CoinSortMode::Amount,
            ApiCoinSortMode::CreatedHeight => CoinSortMode::CreatedHeight,
            ApiCoinSortMode::SpentHeight => CoinSortMode::SpentHeight,
        };
        let filter_mode = match req.filter_mode {
            ApiCoinFilterMode::Owned => CoinFilterMode::Owned,
            ApiCoinFilterMode::Spent => CoinFilterMode::Spent,
            ApiCoinFilterMode::All => CoinFilterMode::All,
            ApiCoinFilterMode::Spendable => CoinFilterMode::Spendable,
        };
        let mut coins = Vec::new();
        let (rows, total) = wallet
            .db
            .coin_records(
                AssetFilter::Id(
                    req.asset_id
                        .map(parse_asset_id)
                        .transpose()?
                        .unwrap_or_default(),
                ),
                req.limit,
                req.offset,
                sort_mode,
                req.ascending,
                filter_mode,
            )
            .await?;

        for row in rows {
            coins.push(CoinRecord {
                coin_id: hex::encode(row.coin.coin_id()),
                address: Address::new(row.coin.puzzle_hash, self.network().prefix()).encode()?,
                amount: Amount::u64(row.coin.amount),
                created_height: row.created_height,
                spent_height: row.spent_height,
                transaction_id: row.mempool_item_hash.map(hex::encode),
                offer_id: row.offer_hash.map(hex::encode),
                created_timestamp: row.created_timestamp,
                spent_timestamp: row.spent_timestamp,
            });
        }

        Ok(GetCoinsResponse { coins, total })
    }

    pub async fn get_all_cats(&self, _req: GetAllCats) -> Result<GetAllCatsResponse> {
        let wallet = self.wallet()?;

        let cats = wallet.db.all_cats().await?;
        let mut records = Vec::with_capacity(cats.len());

        for cat in cats {
            let balance = wallet.db.cat_balance(cat.hash).await?;

            records.push(TokenRecord {
                asset_id: hex::encode(cat.hash),
                name: cat.name,
                ticker: cat.ticker,
                description: cat.description,
                icon_url: cat.icon_url,
                visible: cat.is_visible,
                balance: Amount::u128(balance),
                precision: cat.precision,
                is_xch: false,
            });
        }

        Ok(GetAllCatsResponse { cats: records })
    }

    pub async fn get_cats(&self, _req: GetCats) -> Result<GetCatsResponse> {
        let wallet = self.wallet()?;

        let cats = wallet.db.owned_cats().await?;

        let mut records = Vec::with_capacity(cats.len());

        for cat in cats {
            let balance = wallet.db.cat_balance(cat.hash).await?;

            records.push(TokenRecord {
                asset_id: hex::encode(cat.hash),
                name: cat.name,
                ticker: cat.ticker,
                description: cat.description,
                icon_url: cat.icon_url,
                visible: cat.is_visible,
                balance: Amount::u128(balance),
                precision: cat.precision,
                is_xch: false,
            });
        }

        Ok(GetCatsResponse { cats: records })
    }

    pub async fn get_xch_token(&self, _req: GetXchToken) -> Result<GetXchTokenResponse> {
        let xch = self.get_token(Bytes32::default()).await?;
        let xch = xch.ok_or(Error::InvalidAssetId("XCH asset not found".to_string()))?;

        Ok(GetXchTokenResponse { xch })
    }

    pub async fn get_cat(&self, req: GetCat) -> Result<GetCatResponse> {
        let asset_id = parse_asset_id(req.asset_id)?;
        let cat = self.get_token(asset_id).await?;

        Ok(GetCatResponse { cat })
    }

    pub async fn get_dids(&self, _req: GetDids) -> Result<GetDidsResponse> {
        let wallet = self.wallet()?;

        let mut dids = Vec::new();

        for row in wallet.db.owned_dids().await? {
            dids.push(DidRecord {
                launcher_id: Address::new(row.asset.hash, "did:chia:".to_string()).encode()?,
                name: row.asset.name,
                visible: row.asset.is_visible,
                coin_id: hex::encode(row.coin_row.coin.coin_id()),
                address: Address::new(row.coin_row.p2_puzzle_hash, self.network().prefix())
                    .encode()?,
                amount: Amount::u64(row.coin_row.coin.amount),
                recovery_hash: row.did_info.recovery_list_hash.map(hex::encode),
                created_height: row.coin_row.created_height,
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
            .filter_map(|did| Address::new(did, "did:chia:".to_string()).encode().ok())
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
            .mempool_items()
            .await?
            .into_iter()
            .map(|tx| {
                Result::Ok(PendingTransactionRecord {
                    transaction_id: hex::encode(tx.hash),
                    fee: Amount::u64(tx.fee),
                    submitted_at: Some(tx.submitted_timestamp.to_string()),
                })
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(GetPendingTransactionsResponse { transactions })
    }

    pub async fn get_transaction(&self, req: GetTransaction) -> Result<GetTransactionResponse> {
        let wallet = self.wallet()?;

        let transaction = wallet.db.transaction(req.height).await?;

        let transaction = transaction
            .map(|row| self.transaction_record(row))
            .transpose()?;

        Ok(GetTransactionResponse { transaction })
    }

    pub async fn get_transactions(&self, req: GetTransactions) -> Result<GetTransactionsResponse> {
        let wallet = self.wallet()?;

        let mut transactions = Vec::new();

        let (transaction_records, total) = wallet
            .db
            .transactions(req.find_value, req.ascending, req.limit, req.offset)
            .await?;

        for row in transaction_records {
            let record = self.transaction_record(row)?;
            transactions.push(record);
        }

        Ok(GetTransactionsResponse {
            transactions,
            total,
        })
    }

    pub async fn get_nft_collections(
        &self,
        req: GetNftCollections,
    ) -> Result<GetNftCollectionsResponse> {
        let wallet = self.wallet()?;

        let (collections, total) = wallet
            .db
            .collections(req.limit, req.offset, req.include_hidden)
            .await?;

        let records = collections
            .into_iter()
            .map(|row| {
                Ok(NftCollectionRecord {
                    collection_id: Address::new(row.hash, "col".to_string()).encode()?,
                    did_id: Address::new(row.minter_hash, "did:chia:".to_string()).encode()?,
                    metadata_collection_id: row.uuid,
                    name: row.name,
                    icon: row.icon_url,
                    visible: row.is_visible,
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
                collection_id: Address::new(collection.hash, "col".to_string()).encode()?,
                did_id: Address::new(collection.minter_hash, "did:chia:".to_string()).encode()?,
                metadata_collection_id: collection.uuid,
                visible: collection.is_visible,
                name: collection.name,
                icon: collection.icon_url,
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
                    Some(NftGroupSearch::NoCollection)
                } else {
                    Some(NftGroupSearch::Collection(parse_collection_id(
                        collection_id.clone(),
                    )?))
                }
            }
            (None, Some(minter_did_id), None) => {
                if minter_did_id == "none" {
                    Some(NftGroupSearch::NoMinterDid)
                } else {
                    Some(NftGroupSearch::MinterDid(parse_did_id(
                        minter_did_id.clone(),
                    )?))
                }
            }
            (None, None, Some(owner_did_id)) => {
                if owner_did_id == "none" {
                    Some(NftGroupSearch::NoOwnerDid)
                } else {
                    Some(NftGroupSearch::OwnerDid(parse_did_id(
                        owner_did_id.clone(),
                    )?))
                }
            }
            (None, None, None) => None,
            _ => return Err(Error::InvalidGroup),
        };

        let sort_mode = match req.sort_mode {
            ApiNftSortMode::Recent => NftSortMode::Recent,
            ApiNftSortMode::Name => NftSortMode::Name,
        };

        let (nfts, total) = wallet
            .db
            .owned_nfts(
                req.name,
                group,
                sort_mode,
                req.include_hidden,
                req.limit,
                req.offset,
            )
            .await?;

        for row in nfts {
            records.push(self.nft_record(row)?);
        }

        Ok(GetNftsResponse {
            nfts: records,
            total,
        })
    }

    pub async fn get_nft(&self, req: GetNft) -> Result<GetNftResponse> {
        let wallet = self.wallet()?;

        let nft_id = parse_nft_id(req.nft_id)?;

        let Some(row) = wallet.db.owned_nft(nft_id).await? else {
            return Ok(GetNftResponse { nft: None });
        };

        Ok(GetNftResponse {
            nft: Some(self.nft_record(row)?),
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
            wallet.db.full_file_data(hash).await?
        } else {
            None
        };

        let offchain_metadata = if let Some(hash) = metadata_hash {
            wallet.db.full_file_data(hash).await?
        } else {
            None
        };

        let hash_matches = data.as_ref().is_some_and(|data| data.is_hash_match);
        let metadata_hash_matches = offchain_metadata
            .as_ref()
            .is_some_and(|offchain_metadata| offchain_metadata.is_hash_match);

        Ok(GetNftDataResponse {
            data: Some(NftData {
                blob: data.as_ref().map(|data| BASE64_STANDARD.encode(&data.data)),
                mime_type: data.map(|data| data.mime_type),
                hash_matches,
                metadata_json: offchain_metadata.and_then(|offchain_metadata| {
                    if offchain_metadata.mime_type == "application/json" {
                        String::from_utf8(offchain_metadata.data).ok()
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
                .icon(data_hash)
                .await?
                .map(|icon| BASE64_STANDARD.encode(icon.data)),
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
                .thumbnail(data_hash)
                .await?
                .map(|thumbnail| BASE64_STANDARD.encode(thumbnail.data)),
        })
    }

    async fn get_token(&self, asset_id: Bytes32) -> Result<Option<TokenRecord>> {
        let wallet = self.wallet()?;

        let token = wallet.db.asset(asset_id).await?;
        let balance = wallet.db.token_balance(asset_id).await?;

        let token = token.map(|token| {
            let is_xch = asset_id == Bytes32::default();
            TokenRecord {
                asset_id: if is_xch {
                    "xch".to_string()
                } else {
                    token.hash.to_string()
                },
                name: token.name,
                ticker: token.ticker,
                description: token.description,
                icon_url: token.icon_url,
                visible: token.is_visible,
                balance: Amount::u128(balance),
                precision: token.precision,
                is_xch,
            }
        });

        Ok(token)
    }

    fn nft_record(&self, row: NftRow) -> Result<NftRecord> {
        let mut allocator = Allocator::new();
        let metadata_ptr = row.nft_info.metadata.to_clvm(&mut allocator)?;
        let metadata = NftMetadata::from_clvm(&allocator, metadata_ptr).ok();

        let data_hash = metadata.as_ref().and_then(|m| m.data_hash);
        let metadata_hash = metadata.as_ref().and_then(|m| m.metadata_hash);
        let license_hash = metadata.as_ref().and_then(|m| m.license_hash);

        Ok(NftRecord {
            launcher_id: Address::new(row.asset.hash, "nft".to_string()).encode()?,
            collection_id: Some(
                Address::new(row.nft_info.collection_hash, "col".to_string()).encode()?,
            ),
            collection_name: row.nft_info.collection_name,
            minter_did: row
                .nft_info
                .minter_hash
                .map(|did| Address::new(did, "did:chia:".to_string()).encode())
                .transpose()?,
            owner_did: row
                .nft_info
                .owner_hash
                .map(|did| Address::new(did, "did:chia:".to_string()).encode())
                .transpose()?,
            visible: row.asset.is_visible,
            name: row.asset.name,
            sensitive_content: row.asset.is_sensitive_content,
            coin_id: hex::encode(row.coin_row.coin.coin_id()),
            address: Address::new(row.coin_row.p2_puzzle_hash, self.network().prefix()).encode()?,
            royalty_address: Address::new(
                row.nft_info.royalty_puzzle_hash,
                self.network().prefix(),
            )
            .encode()?,
            royalty_ten_thousandths: row.nft_info.royalty_basis_points,
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
            edition_number: metadata.as_ref().map(|m| m.edition_number as u32),
            edition_total: metadata.as_ref().map(|m| m.edition_total as u32),
            created_height: row.coin_row.created_height,
        })
    }

    fn transaction_coin(&self, transaction_coin: TransactionCoin) -> Result<TransactionCoinRecord> {
        Ok(TransactionCoinRecord {
            coin_id: hex::encode(transaction_coin.coin.coin_id()),
            address: transaction_coin
                .p2_puzzle_hash
                .map(|p2_puzzle_hash| {
                    Address::new(p2_puzzle_hash, self.network().prefix()).encode()
                })
                .transpose()?,
            address_kind: address_kind(transaction_coin.p2_puzzle_hash),
            amount: Amount::u64(transaction_coin.coin.amount),
            asset: encode_asset(transaction_coin.asset)?,
        })
    }

    fn transaction_record(&self, transaction: Transaction) -> Result<TransactionRecord> {
        let mut spent = Vec::new();
        let mut created = Vec::new();

        for coin in transaction.created {
            created.push(self.transaction_coin(coin)?);
        }
        for coin in transaction.spent {
            spent.push(self.transaction_coin(coin)?);
        }

        Ok(TransactionRecord {
            height: transaction.height,
            timestamp: transaction.timestamp,
            spent,
            created,
        })
    }
}
