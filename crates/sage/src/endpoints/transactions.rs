use std::time::Duration;

use chia::{
    protocol::{Bytes, Bytes32, CoinSpend},
    puzzles::nft::NftMetadata,
};
use chia_wallet_sdk::{driver::MetadataUpdate, utils::Address};
use itertools::Itertools;
use sage_api::{
    AddNftUri, AssignNftsToDid, AutoCombineCat, AutoCombineCatResponse, AutoCombineXch,
    AutoCombineXchResponse, BulkMintNfts, BulkMintNftsResponse, BulkSendCat, BulkSendXch,
    CombineCat, CombineXch, CreateDid, IssueCat, MultiSend, NftUriKind, NormalizeDids, SendCat,
    SendXch, SignCoinSpends, SignCoinSpendsResponse, SplitCat, SplitXch, SubmitTransaction,
    SubmitTransactionResponse, TransactionResponse, TransferDids, TransferNfts, ViewCoinSpends,
    ViewCoinSpendsResponse,
};
use sage_assets::fetch_uris_without_hash;
use sage_database::CatRow;
use sage_wallet::{MultiSendPayment, WalletNftMint};
use tokio::time::timeout;

use crate::{
    fetch_cats, json_bundle, json_spend, parse_amount, parse_asset_id, parse_coin_id, parse_did_id,
    parse_hash, parse_memos, parse_nft_id, rust_bundle, rust_spend, ConfirmationInfo, Error,
    Result, Sage,
};

impl Sage {
    pub async fn send_xch(&self, req: SendXch) -> Result<TransactionResponse> {
        let wallet = self.wallet()?;
        let puzzle_hash = self.parse_address(req.address)?;
        let amount = parse_amount(req.amount)?;
        let fee = parse_amount(req.fee)?;
        let memos = parse_memos(req.memos)?;

        let coin_spends = wallet
            .send_xch(vec![(puzzle_hash, amount)], fee, memos)
            .await?;
        self.transact(coin_spends, req.auto_submit).await
    }

    pub async fn bulk_send_xch(&self, req: BulkSendXch) -> Result<TransactionResponse> {
        let wallet = self.wallet()?;

        let amount = parse_amount(req.amount)?;

        let mut amounts = Vec::with_capacity(req.addresses.len());

        for address in req.addresses {
            amounts.push((self.parse_address(address)?, amount));
        }

        let fee = parse_amount(req.fee)?;
        let memos = parse_memos(req.memos)?;

        let coin_spends = wallet.send_xch(amounts, fee, memos).await?;
        self.transact(coin_spends, req.auto_submit).await
    }

    pub async fn combine_xch(&self, req: CombineXch) -> Result<TransactionResponse> {
        let wallet = self.wallet()?;
        let fee = parse_amount(req.fee)?;
        let coin_ids = req
            .coin_ids
            .into_iter()
            .map(parse_coin_id)
            .collect::<Result<Vec<Bytes32>>>()?;

        let coin_spends = wallet.combine_xch(coin_ids, fee).await?;
        self.transact(coin_spends, req.auto_submit).await
    }

    pub async fn auto_combine_xch(&self, req: AutoCombineXch) -> Result<AutoCombineXchResponse> {
        let wallet = self.wallet()?;
        let fee = parse_amount(req.fee)?;
        let max_amount = req.max_coin_amount.map(parse_amount).transpose()?;

        let coin_ids = wallet
            .db
            .spendable_coins()
            .await?
            .into_iter()
            .filter(|coin| {
                let Some(max_amount) = max_amount else {
                    return true;
                };
                coin.amount <= max_amount
            })
            .sorted_by_key(|coin| coin.amount)
            .take(req.max_coins as usize)
            .map(|coin| coin.coin_id())
            .collect_vec();

        let coin_ids_hex = coin_ids.iter().map(hex::encode).collect_vec();
        let coin_spends = wallet.combine_xch(coin_ids, fee).await?;
        let response = self.transact(coin_spends, req.auto_submit).await?;

        Ok(AutoCombineXchResponse {
            coin_ids: coin_ids_hex,
            summary: response.summary,
            coin_spends: response.coin_spends,
        })
    }

    pub async fn split_xch(&self, req: SplitXch) -> Result<TransactionResponse> {
        let wallet = self.wallet()?;
        let fee = parse_amount(req.fee)?;

        let coin_ids = req
            .coin_ids
            .into_iter()
            .map(parse_coin_id)
            .collect::<Result<Vec<Bytes32>>>()?;

        let coin_spends = wallet
            .split_xch(coin_ids, req.output_count as usize, fee)
            .await?;
        self.transact(coin_spends, req.auto_submit).await
    }

    pub async fn combine_cat(&self, req: CombineCat) -> Result<TransactionResponse> {
        let wallet = self.wallet()?;
        let fee = parse_amount(req.fee)?;
        let cats = fetch_cats(&wallet, req.coin_ids).await?;

        let coin_spends = wallet.combine_cat(cats, fee, false, true).await?;
        self.transact(coin_spends, req.auto_submit).await
    }

    pub async fn auto_combine_cat(&self, req: AutoCombineCat) -> Result<AutoCombineCatResponse> {
        let wallet = self.wallet()?;
        let fee = parse_amount(req.fee)?;
        let asset_id = parse_asset_id(req.asset_id)?;
        let max_amount = req.max_coin_amount.map(parse_amount).transpose()?;

        let rows = wallet
            .db
            .spendable_cat_coins(asset_id)
            .await?
            .into_iter()
            .filter(|cat| {
                let Some(max_amount) = max_amount else {
                    return true;
                };
                cat.coin.amount <= max_amount
            })
            .sorted_by_key(|cat| cat.coin.amount)
            .take(req.max_coins as usize)
            .collect_vec();

        let mut cats = Vec::with_capacity(rows.len());

        for row in rows {
            let Some(cat) = wallet.db.cat_coin(row.coin.coin_id()).await? else {
                return Err(Error::MissingCatCoin(row.coin.coin_id()));
            };
            cats.push(cat);
        }

        let coin_ids = cats
            .iter()
            .map(|cat| hex::encode(cat.coin.coin_id()))
            .collect_vec();

        let coin_spends = wallet.combine_cat(cats, fee, false, true).await?;
        let response = self.transact(coin_spends, req.auto_submit).await?;

        Ok(AutoCombineCatResponse {
            coin_ids,
            summary: response.summary,
            coin_spends: response.coin_spends,
        })
    }

    pub async fn split_cat(&self, req: SplitCat) -> Result<TransactionResponse> {
        let wallet = self.wallet()?;
        let fee = parse_amount(req.fee)?;
        let cats = fetch_cats(&wallet, req.coin_ids).await?;

        let coin_spends = wallet
            .split_cat(cats, req.output_count as usize, fee, false, true)
            .await?;
        self.transact(coin_spends, req.auto_submit).await
    }

    pub async fn issue_cat(&self, req: IssueCat) -> Result<TransactionResponse> {
        let wallet = self.wallet()?;
        let amount = parse_amount(req.amount)?;
        let fee = parse_amount(req.fee)?;

        let (coin_spends, asset_id) = wallet.issue_cat(amount, fee, false, true).await?;
        wallet
            .db
            .insert_cat(CatRow {
                asset_id,
                name: Some(req.name),
                ticker: Some(req.ticker),
                description: None,
                icon: None,
                visible: true,
                fetched: true,
            })
            .await?;

        self.transact(coin_spends, req.auto_submit).await
    }

    pub async fn send_cat(&self, req: SendCat) -> Result<TransactionResponse> {
        let wallet = self.wallet()?;
        let asset_id = parse_asset_id(req.asset_id)?;
        let puzzle_hash = self.parse_address(req.address)?;
        let amount = parse_amount(req.amount)?;
        let fee = parse_amount(req.fee)?;
        let memos = parse_memos(req.memos)?;

        let coin_spends = wallet
            .send_cat(
                asset_id,
                vec![(puzzle_hash, amount)],
                fee,
                req.include_hint,
                memos,
            )
            .await?;
        self.transact(coin_spends, req.auto_submit).await
    }

    pub async fn bulk_send_cat(&self, req: BulkSendCat) -> Result<TransactionResponse> {
        let wallet = self.wallet()?;
        let asset_id = parse_asset_id(req.asset_id)?;

        let amount = parse_amount(req.amount)?;

        let mut amounts = Vec::with_capacity(req.addresses.len());

        for address in req.addresses {
            amounts.push((self.parse_address(address)?, amount));
        }

        let fee = parse_amount(req.fee)?;
        let memos = parse_memos(req.memos)?;

        let coin_spends = wallet
            .send_cat(asset_id, amounts, fee, req.include_hint, memos)
            .await?;
        self.transact(coin_spends, req.auto_submit).await
    }

    pub async fn multi_send(&self, req: MultiSend) -> Result<TransactionResponse> {
        let wallet = self.wallet()?;

        let mut payments = Vec::with_capacity(req.payments.len());

        for payment in req.payments {
            let asset_id = if let Some(asset_id) = payment.asset_id {
                Some(parse_asset_id(asset_id)?)
            } else {
                None
            };
            let amount = parse_amount(payment.amount)?;
            let puzzle_hash = self.parse_address(payment.address)?;
            let memos = if let Some(list) = payment.memos {
                let mut memos = Vec::new();
                for memo in list {
                    memos.push(Bytes::from(hex::decode(memo)?));
                }
                Some(memos)
            } else {
                None
            };

            payments.push(MultiSendPayment {
                asset_id,
                amount,
                puzzle_hash,
                memos,
            });
        }

        let fee = parse_amount(req.fee)?;

        let coin_spends = wallet.multi_send(payments, fee, false, true).await?;
        self.transact(coin_spends, req.auto_submit).await
    }

    pub async fn create_did(&self, req: CreateDid) -> Result<TransactionResponse> {
        let wallet = self.wallet()?;
        let fee = parse_amount(req.fee)?;

        let (coin_spends, did) = wallet.create_did(fee, false, true).await?;
        wallet
            .db
            .set_future_did_name(did.info.launcher_id, req.name.clone())
            .await?;

        let mut info = ConfirmationInfo::default();
        info.did_names.insert(did.info.launcher_id, req.name);
        self.transact_with(coin_spends, req.auto_submit, info).await
    }

    pub async fn bulk_mint_nfts(&self, req: BulkMintNfts) -> Result<BulkMintNftsResponse> {
        let wallet = self.wallet()?;
        let fee = parse_amount(req.fee)?;
        let did_id = parse_did_id(req.did_id)?;

        let mut mints = Vec::with_capacity(req.mints.len());
        let mut info = ConfirmationInfo::default();

        for item in req.mints {
            let royalty_puzzle_hash = item
                .royalty_address
                .map(|address| self.parse_address(address))
                .transpose()?;

            let royalty_ten_thousandths = item.royalty_ten_thousandths;

            let data_hash = if let Some(data_hash) = item.data_hash {
                Some(parse_hash(data_hash)?)
            } else if item.data_uris.is_empty() {
                None
            } else {
                let data = timeout(
                    Duration::from_secs(10),
                    fetch_uris_without_hash(item.data_uris.clone()),
                )
                .await??;

                let hash = data.hash;
                info.nft_data.insert(hash, data);

                Some(hash)
            };

            let metadata_hash = if let Some(metadata_hash) = item.metadata_hash {
                Some(parse_hash(metadata_hash)?)
            } else if item.metadata_uris.is_empty() {
                None
            } else {
                let metadata = timeout(
                    Duration::from_secs(10),
                    fetch_uris_without_hash(item.metadata_uris.clone()),
                )
                .await??;

                let hash = metadata.hash;
                info.nft_data.insert(hash, metadata);

                Some(hash)
            };

            let license_hash = if let Some(license_hash) = item.license_hash {
                Some(parse_hash(license_hash)?)
            } else if item.license_uris.is_empty() {
                None
            } else {
                let data = timeout(
                    Duration::from_secs(10),
                    fetch_uris_without_hash(item.license_uris.clone()),
                )
                .await??;

                let hash = data.hash;
                info.nft_data.insert(hash, data);

                Some(hash)
            };

            let p2_puzzle_hash = if let Some(address) = item.address {
                Some(self.parse_address(address)?)
            } else {
                None
            };

            mints.push(WalletNftMint {
                metadata: NftMetadata {
                    edition_number: item.edition_number.map_or(1, Into::into),
                    edition_total: item.edition_total.map_or(1, Into::into),
                    data_uris: item.data_uris,
                    data_hash,
                    metadata_uris: item.metadata_uris,
                    metadata_hash,
                    license_uris: item.license_uris,
                    license_hash,
                },
                p2_puzzle_hash,
                royalty_puzzle_hash,
                royalty_ten_thousandths,
            });
        }

        let (coin_spends, nfts, _did) = wallet
            .bulk_mint_nfts(fee, did_id, mints, false, true)
            .await?;

        let mut nft_ids = Vec::with_capacity(nfts.len());

        for nft in nfts {
            nft_ids.push(Address::new(nft.info.launcher_id, "nft".to_string()).encode()?);
        }

        let response = self
            .transact_with(coin_spends, req.auto_submit, info)
            .await?;

        Ok(BulkMintNftsResponse {
            nft_ids,
            summary: response.summary,
            coin_spends: response.coin_spends,
        })
    }

    pub async fn transfer_nfts(&self, req: TransferNfts) -> Result<TransactionResponse> {
        let wallet = self.wallet()?;
        let nft_ids = req
            .nft_ids
            .into_iter()
            .map(parse_nft_id)
            .collect::<Result<Vec<_>>>()?;
        let puzzle_hash = self.parse_address(req.address)?;
        let fee = parse_amount(req.fee)?;

        let coin_spends = wallet
            .transfer_nfts(nft_ids, puzzle_hash, fee, false, true)
            .await?;
        self.transact(coin_spends, req.auto_submit).await
    }

    pub async fn add_nft_uri(&self, req: AddNftUri) -> Result<TransactionResponse> {
        let wallet = self.wallet()?;
        let nft_id = parse_nft_id(req.nft_id)?;
        let fee = parse_amount(req.fee)?;

        let uri = match req.kind {
            NftUriKind::Data => MetadataUpdate::NewDataUri(req.uri),
            NftUriKind::Metadata => MetadataUpdate::NewMetadataUri(req.uri),
            NftUriKind::License => MetadataUpdate::NewLicenseUri(req.uri),
        };

        let (coin_spends, _new_nft) = wallet.add_nft_uri(nft_id, fee, uri, false, true).await?;
        self.transact(coin_spends, req.auto_submit).await
    }

    pub async fn assign_nfts_to_did(&self, req: AssignNftsToDid) -> Result<TransactionResponse> {
        let wallet = self.wallet()?;
        let nft_ids = req
            .nft_ids
            .into_iter()
            .map(parse_nft_id)
            .collect::<Result<Vec<_>>>()?;
        let did_id = req.did_id.map(parse_did_id).transpose()?;
        let fee = parse_amount(req.fee)?;

        let coin_spends = wallet
            .assign_nfts(nft_ids, did_id, fee, false, true)
            .await?;
        self.transact(coin_spends, req.auto_submit).await
    }

    pub async fn transfer_dids(&self, req: TransferDids) -> Result<TransactionResponse> {
        let wallet = self.wallet()?;
        let did_ids = req
            .did_ids
            .into_iter()
            .map(parse_did_id)
            .collect::<Result<Vec<_>>>()?;
        let puzzle_hash = self.parse_address(req.address)?;
        let fee = parse_amount(req.fee)?;

        let coin_spends = wallet
            .transfer_dids(did_ids, puzzle_hash, fee, false, true)
            .await?;
        self.transact(coin_spends, req.auto_submit).await
    }

    pub async fn normalize_dids(&self, req: NormalizeDids) -> Result<TransactionResponse> {
        let wallet = self.wallet()?;
        let did_ids = req
            .did_ids
            .into_iter()
            .map(parse_did_id)
            .collect::<Result<Vec<_>>>()?;
        let fee = parse_amount(req.fee)?;

        let coin_spends = wallet.normalize_dids(did_ids, fee, false, true).await?;
        self.transact(coin_spends, req.auto_submit).await
    }

    pub async fn sign_coin_spends(&self, req: SignCoinSpends) -> Result<SignCoinSpendsResponse> {
        let coin_spends = req
            .coin_spends
            .into_iter()
            .map(rust_spend)
            .collect::<Result<Vec<_>>>()?;
        let spend_bundle = self.sign(coin_spends, req.partial).await?;
        let json_bundle = json_bundle(&spend_bundle);

        if req.auto_submit {
            self.submit(spend_bundle).await?;
        }

        Ok(SignCoinSpendsResponse {
            spend_bundle: json_bundle,
        })
    }

    pub async fn view_coin_spends(&self, req: ViewCoinSpends) -> Result<ViewCoinSpendsResponse> {
        let coin_spends = req
            .coin_spends
            .into_iter()
            .map(rust_spend)
            .collect::<Result<Vec<_>>>()?;

        Ok(ViewCoinSpendsResponse {
            summary: self
                .summarize(coin_spends, ConfirmationInfo::default())
                .await?,
        })
    }

    pub async fn submit_transaction(
        &self,
        req: SubmitTransaction,
    ) -> Result<SubmitTransactionResponse> {
        let spend_bundle = rust_bundle(req.spend_bundle)?;
        self.submit(spend_bundle).await?;

        Ok(SubmitTransactionResponse {})
    }

    pub(crate) async fn transact(
        &self,
        coin_spends: Vec<CoinSpend>,
        auto_submit: bool,
    ) -> Result<TransactionResponse> {
        self.transact_with(coin_spends, auto_submit, ConfirmationInfo::default())
            .await
    }

    async fn transact_with(
        &self,
        coin_spends: Vec<CoinSpend>,
        auto_submit: bool,
        info: ConfirmationInfo,
    ) -> Result<TransactionResponse> {
        if auto_submit {
            let spend_bundle = self.sign(coin_spends.clone(), false).await?;
            self.submit(spend_bundle).await?;
        }

        let json_spends = coin_spends.iter().map(json_spend).collect();

        Ok(TransactionResponse {
            summary: self.summarize(coin_spends, info).await?,
            coin_spends: json_spends,
        })
    }
}
