use std::time::Duration;

use chia::{
    protocol::{Bytes, CoinSpend},
    puzzles::nft::NftMetadata,
};
use chia_wallet_sdk::MetadataUpdate;
use sage_api::{
    AddNftUri, AssignNftsToDid, BulkMintNfts, CombineCat, CombineXch, CreateDid, IssueCat,
    NftUriKind, SendCat, SendXch, SignCoinSpends, SignCoinSpendsResponse, SplitCat, SplitXch,
    SubmitTransaction, SubmitTransactionResponse, TransactionResponse, TransferDids, TransferNfts,
    ViewCoinSpends, ViewCoinSpendsResponse,
};
use sage_database::CatRow;
use sage_wallet::{fetch_uris, WalletNftMint};

use crate::{
    fetch_cats, fetch_coins, json_bundle, json_spend, parse_asset_id, parse_cat_amount,
    parse_did_id, parse_nft_id, rust_bundle, rust_spend, ConfirmationInfo, Result, Sage,
};

impl Sage {
    pub async fn send_xch(&self, req: SendXch) -> Result<TransactionResponse> {
        let wallet = self.wallet()?;
        let puzzle_hash = self.parse_address(req.address)?;
        let amount = self.parse_amount(req.amount)?;
        let fee = self.parse_amount(req.fee)?;

        let mut memos = Vec::new();

        for memo in req.memos {
            memos.push(Bytes::from(hex::decode(memo)?));
        }

        let coin_spends = wallet
            .send_xch(puzzle_hash, amount, fee, memos, false, true)
            .await?;
        self.transact(coin_spends, req.auto_submit).await
    }

    pub async fn combine_xch(&self, req: CombineXch) -> Result<TransactionResponse> {
        let wallet = self.wallet()?;
        let fee = self.parse_amount(req.fee)?;
        let coins = fetch_coins(&wallet, req.coin_ids).await?;

        let coin_spends = wallet.combine_xch(coins, fee, false, true).await?;
        self.transact(coin_spends, req.auto_submit).await
    }

    pub async fn split_xch(&self, req: SplitXch) -> Result<TransactionResponse> {
        let wallet = self.wallet()?;
        let fee = self.parse_amount(req.fee)?;
        let coins = fetch_coins(&wallet, req.coin_ids).await?;

        let coin_spends = wallet
            .split_xch(&coins, req.output_count as usize, fee, false, true)
            .await?;
        self.transact(coin_spends, req.auto_submit).await
    }

    pub async fn combine_cat(&self, req: CombineCat) -> Result<TransactionResponse> {
        let wallet = self.wallet()?;
        let fee = self.parse_amount(req.fee)?;
        let cats = fetch_cats(&wallet, req.coin_ids).await?;

        let coin_spends = wallet.combine_cat(cats, fee, false, true).await?;
        self.transact(coin_spends, req.auto_submit).await
    }

    pub async fn split_cat(&self, req: SplitCat) -> Result<TransactionResponse> {
        let wallet = self.wallet()?;
        let fee = self.parse_amount(req.fee)?;
        let cats = fetch_cats(&wallet, req.coin_ids).await?;

        let coin_spends = wallet
            .split_cat(cats, req.output_count as usize, fee, false, true)
            .await?;
        self.transact(coin_spends, req.auto_submit).await
    }

    pub async fn issue_cat(&self, req: IssueCat) -> Result<TransactionResponse> {
        let wallet = self.wallet()?;
        let amount = parse_cat_amount(req.amount)?;
        let fee = self.parse_amount(req.fee)?;

        let (coin_spends, asset_id) = wallet.issue_cat(amount, fee, None, false, true).await?;
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
        let amount = parse_cat_amount(req.amount)?;
        let fee = self.parse_amount(req.fee)?;

        let coin_spends = wallet
            .send_cat(asset_id, puzzle_hash, amount, fee, false, true)
            .await?;
        self.transact(coin_spends, req.auto_submit).await
    }

    pub async fn create_did(&self, req: CreateDid) -> Result<TransactionResponse> {
        let wallet = self.wallet()?;
        let fee = self.parse_amount(req.fee)?;

        let (coin_spends, did) = wallet.create_did(fee, false, true).await?;
        wallet
            .db
            .set_future_did_name(did.info.launcher_id, req.name.clone())
            .await?;

        let mut info = ConfirmationInfo::default();
        info.did_names.insert(did.info.launcher_id, req.name);
        self.transact_with(coin_spends, req.auto_submit, info).await
    }

    pub async fn bulk_mint_nfts(&self, req: BulkMintNfts) -> Result<TransactionResponse> {
        let wallet = self.wallet()?;
        let fee = self.parse_amount(req.fee)?;
        let did_id = parse_did_id(req.did_id)?;

        let mut mints = Vec::with_capacity(req.mints.len());
        let mut info = ConfirmationInfo::default();

        for item in req.mints {
            let royalty_puzzle_hash = item
                .royalty_address
                .map(|address| self.parse_address(address))
                .transpose()?;

            let royalty_ten_thousandths = item.royalty_ten_thousandths;

            let data_hash = if item.data_uris.is_empty() {
                None
            } else {
                let data = fetch_uris(
                    item.data_uris.clone(),
                    Duration::from_secs(15),
                    Duration::from_secs(5),
                )
                .await?;

                let hash = data.hash;
                info.nft_data.insert(hash, data);

                Some(hash)
            };

            let metadata_hash = if item.metadata_uris.is_empty() {
                None
            } else {
                let metadata = fetch_uris(
                    item.metadata_uris.clone(),
                    Duration::from_secs(15),
                    Duration::from_secs(15),
                )
                .await?;

                let hash = metadata.hash;
                info.nft_data.insert(hash, metadata);

                Some(hash)
            };

            let license_hash = if item.license_uris.is_empty() {
                None
            } else {
                let data = fetch_uris(
                    item.license_uris.clone(),
                    Duration::from_secs(15),
                    Duration::from_secs(15),
                )
                .await?;

                let hash = data.hash;
                info.nft_data.insert(hash, data);

                Some(hash)
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
                royalty_puzzle_hash,
                royalty_ten_thousandths,
            });
        }

        let (coin_spends, _nfts, _did) = wallet
            .bulk_mint_nfts(fee, did_id, mints, false, true)
            .await?;
        self.transact_with(coin_spends, req.auto_submit, info).await
    }

    pub async fn transfer_nfts(&self, req: TransferNfts) -> Result<TransactionResponse> {
        let wallet = self.wallet()?;
        let nft_ids = req
            .nft_ids
            .into_iter()
            .map(parse_nft_id)
            .collect::<Result<Vec<_>>>()?;
        let puzzle_hash = self.parse_address(req.address)?;
        let fee = self.parse_amount(req.fee)?;

        let coin_spends = wallet
            .transfer_nfts(nft_ids, puzzle_hash, fee, false, true)
            .await?;
        self.transact(coin_spends, req.auto_submit).await
    }

    pub async fn add_nft_uri(&self, req: AddNftUri) -> Result<TransactionResponse> {
        let wallet = self.wallet()?;
        let nft_id = parse_nft_id(req.nft_id)?;
        let fee = self.parse_amount(req.fee)?;

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
        let fee = self.parse_amount(req.fee)?;

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
        let fee = self.parse_amount(req.fee)?;

        let coin_spends = wallet
            .transfer_dids(did_ids, puzzle_hash, fee, false, true)
            .await?;
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

    async fn transact(
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
