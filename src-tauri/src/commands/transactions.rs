use std::{collections::HashMap, time::Duration};

use base64::prelude::*;
use chia::{
    bls::Signature,
    protocol::{Bytes32, Coin, CoinSpend, SpendBundle},
    puzzles::nft::NftMetadata,
};
use chia_wallet_sdk::{
    decode_address, encode_address, AggSigConstants, MAINNET_CONSTANTS, TESTNET11_CONSTANTS,
};
use sage_api::{
    Amount, BulkMintNfts, BulkMintNftsResponse, CoinJson, CoinSpendJson, Input, InputKind, Output,
    SpendBundleJson, TransactionSummary,
};
use sage_database::{CatRow, Database, NftRow};
use sage_wallet::{
    compute_nft_info, fetch_uris, insert_transaction, ChildKind, CoinKind, Data, Transaction,
    Wallet, WalletError, WalletNftMint,
};
use specta::specta;
use tauri::{command, State};
use tokio::sync::MutexGuard;

use crate::{
    app_state::{AppState, AppStateInner},
    error::{Error, Result},
};

#[command]
#[specta]
pub async fn validate_address(state: State<'_, AppState>, address: String) -> Result<bool> {
    let state = state.lock().await;
    let Some((_puzzle_hash, prefix)) = decode_address(&address).ok() else {
        return Ok(false);
    };
    Ok(prefix == state.network().address_prefix)
}

#[command]
#[specta]
pub async fn send(
    state: State<'_, AppState>,
    address: String,
    amount: Amount,
    fee: Amount,
) -> Result<TransactionSummary> {
    let state = state.lock().await;
    let wallet = state.wallet()?;

    if !state.keychain.has_secret_key(wallet.fingerprint) {
        return Err(Error::no_secret_key());
    }

    let (puzzle_hash, prefix) = decode_address(&address)?;
    if prefix != state.network().address_prefix {
        return Err(Error::invalid_prefix(&prefix));
    }

    let Some(amount) = amount.to_mojos(state.unit.decimals) else {
        return Err(Error::invalid_amount(&amount));
    };

    let Some(fee) = fee.to_mojos(state.unit.decimals) else {
        return Err(Error::invalid_amount(&fee));
    };

    let coin_spends = wallet
        .send_xch(puzzle_hash.into(), amount, fee, Vec::new(), false, true)
        .await?;

    summarize(&state, &wallet, coin_spends, ConfirmationInfo::default()).await
}

#[command]
#[specta]
pub async fn combine(
    state: State<'_, AppState>,
    coin_ids: Vec<String>,
    fee: Amount,
) -> Result<TransactionSummary> {
    let state = state.lock().await;
    let wallet = state.wallet()?;

    if !state.keychain.has_secret_key(wallet.fingerprint) {
        return Err(Error::no_secret_key());
    }

    let Some(fee) = fee.to_mojos(state.unit.decimals) else {
        return Err(Error::invalid_amount(&fee));
    };

    let coin_ids = coin_ids
        .iter()
        .map(|coin_id| Ok(hex::decode(coin_id)?.try_into()?))
        .collect::<Result<Vec<Bytes32>>>()?;

    let mut tx = wallet.db.tx().await?;

    let mut coins = Vec::new();

    for coin_id in coin_ids {
        let Some(coin_state) = tx.coin_state(coin_id).await? else {
            return Err(Error::unknown_coin_id());
        };

        if coin_state.spent_height.is_some() {
            return Err(Error::already_spent(coin_id));
        }

        coins.push(coin_state.coin);
    }

    tx.commit().await?;

    let coin_spends = wallet.combine_xch(coins, fee, false, true).await?;

    summarize(&state, &wallet, coin_spends, ConfirmationInfo::default()).await
}

#[command]
#[specta]
pub async fn split(
    state: State<'_, AppState>,
    coin_ids: Vec<String>,
    output_count: u32,
    fee: Amount,
) -> Result<TransactionSummary> {
    let state = state.lock().await;
    let wallet = state.wallet()?;

    if !state.keychain.has_secret_key(wallet.fingerprint) {
        return Err(Error::no_secret_key());
    }

    let Some(fee) = fee.to_mojos(state.unit.decimals) else {
        return Err(Error::invalid_amount(&fee));
    };

    let coin_ids = coin_ids
        .iter()
        .map(|coin_id| Ok(hex::decode(coin_id)?.try_into()?))
        .collect::<Result<Vec<Bytes32>>>()?;

    let mut tx = wallet.db.tx().await?;

    let mut coins = Vec::new();

    for coin_id in coin_ids {
        let Some(coin_state) = tx.coin_state(coin_id).await? else {
            return Err(Error::unknown_coin_id());
        };

        if coin_state.spent_height.is_some() {
            return Err(Error::already_spent(coin_id));
        }

        coins.push(coin_state.coin);
    }

    tx.commit().await?;

    let coin_spends = wallet
        .split_xch(&coins, output_count as usize, fee, false, true)
        .await?;

    summarize(&state, &wallet, coin_spends, ConfirmationInfo::default()).await
}

#[command]
#[specta]
pub async fn combine_cat(
    state: State<'_, AppState>,
    coin_ids: Vec<String>,
    fee: Amount,
) -> Result<TransactionSummary> {
    let state = state.lock().await;
    let wallet = state.wallet()?;

    if !state.keychain.has_secret_key(wallet.fingerprint) {
        return Err(Error::no_secret_key());
    }

    let Some(fee) = fee.to_mojos(state.unit.decimals) else {
        return Err(Error::invalid_amount(&fee));
    };

    let coin_ids = coin_ids
        .iter()
        .map(|coin_id| Ok(hex::decode(coin_id)?.try_into()?))
        .collect::<Result<Vec<Bytes32>>>()?;

    let mut cats = Vec::new();

    for coin_id in coin_ids {
        let Some(cat) = wallet.db.cat_coin(coin_id).await? else {
            return Err(Error::unknown_coin_id());
        };
        cats.push(cat);
    }

    let coin_spends = wallet.combine_cat(cats, fee, false, true).await?;

    summarize(&state, &wallet, coin_spends, ConfirmationInfo::default()).await
}

#[command]
#[specta]
pub async fn split_cat(
    state: State<'_, AppState>,
    coin_ids: Vec<String>,
    output_count: u32,
    fee: Amount,
) -> Result<TransactionSummary> {
    let state = state.lock().await;
    let wallet = state.wallet()?;

    if !state.keychain.has_secret_key(wallet.fingerprint) {
        return Err(Error::no_secret_key());
    }

    let Some(fee) = fee.to_mojos(state.unit.decimals) else {
        return Err(Error::invalid_amount(&fee));
    };

    let coin_ids = coin_ids
        .iter()
        .map(|coin_id| Ok(hex::decode(coin_id)?.try_into()?))
        .collect::<Result<Vec<Bytes32>>>()?;

    let mut cats = Vec::new();

    for coin_id in coin_ids {
        let Some(cat) = wallet.db.cat_coin(coin_id).await? else {
            return Err(Error::unknown_coin_id());
        };
        cats.push(cat);
    }

    let coin_spends = wallet
        .split_cat(cats, output_count as usize, fee, false, true)
        .await?;

    summarize(&state, &wallet, coin_spends, ConfirmationInfo::default()).await
}

#[command]
#[specta]
pub async fn issue_cat(
    state: State<'_, AppState>,
    name: String,
    ticker: String,
    amount: Amount,
    fee: Amount,
) -> Result<TransactionSummary> {
    let state = state.lock().await;
    let wallet = state.wallet()?;

    if !state.keychain.has_secret_key(wallet.fingerprint) {
        return Err(Error::no_secret_key());
    }

    let Some(amount) = amount.to_mojos(3) else {
        return Err(Error::invalid_amount(&amount));
    };

    let Some(fee) = fee.to_mojos(state.unit.decimals) else {
        return Err(Error::invalid_amount(&fee));
    };

    let (coin_spends, asset_id) = wallet.issue_cat(amount, fee, None, false, true).await?;

    let summary = summarize(&state, &wallet, coin_spends, ConfirmationInfo::default()).await?;

    wallet
        .db
        .maybe_insert_cat(CatRow {
            asset_id,
            name: Some(name),
            ticker: Some(ticker),
            description: None,
            icon_url: None,
            visible: true,
        })
        .await?;

    Ok(summary)
}

#[command]
#[specta]
pub async fn send_cat(
    state: State<'_, AppState>,
    asset_id: String,
    address: String,
    amount: Amount,
    fee: Amount,
) -> Result<TransactionSummary> {
    let state = state.lock().await;
    let wallet = state.wallet()?;

    if !state.keychain.has_secret_key(wallet.fingerprint) {
        return Err(Error::no_secret_key());
    }

    let asset_id: Bytes32 = hex::decode(asset_id)?.try_into()?;

    let (puzzle_hash, prefix) = decode_address(&address)?;
    if prefix != state.network().address_prefix {
        return Err(Error::invalid_prefix(&prefix));
    }

    let Some(amount) = amount.to_mojos(3) else {
        return Err(Error::invalid_amount(&amount));
    };

    let Some(fee) = fee.to_mojos(state.unit.decimals) else {
        return Err(Error::invalid_amount(&fee));
    };

    let coin_spends = wallet
        .send_cat(asset_id, puzzle_hash.into(), amount, fee, false, true)
        .await?;

    summarize(&state, &wallet, coin_spends, ConfirmationInfo::default()).await
}

#[command]
#[specta]
pub async fn create_did(
    state: State<'_, AppState>,
    name: String,
    fee: Amount,
) -> Result<TransactionSummary> {
    let state = state.lock().await;
    let wallet = state.wallet()?;

    if !state.keychain.has_secret_key(wallet.fingerprint) {
        return Err(Error::no_secret_key());
    }

    let Some(fee) = fee.to_mojos(state.unit.decimals) else {
        return Err(Error::invalid_amount(&fee));
    };

    let (coin_spends, did) = wallet.create_did(fee, false, true).await?;

    wallet
        .db
        .insert_new_did(did.info.launcher_id, Some(name.clone()), true)
        .await?;

    let mut confirm_info = ConfirmationInfo::default();
    confirm_info.did_names.insert(did.info.launcher_id, name);

    summarize(&state, &wallet, coin_spends, confirm_info).await
}

#[command]
#[specta]
pub async fn bulk_mint_nfts(
    state: State<'_, AppState>,
    request: BulkMintNfts,
) -> Result<BulkMintNftsResponse> {
    let state = state.lock().await;
    let wallet = state.wallet()?;

    if !state.keychain.has_secret_key(wallet.fingerprint) {
        return Err(Error::no_secret_key());
    }

    let Some(fee) = request.fee.to_mojos(state.unit.decimals) else {
        return Err(Error::invalid_amount(&request.fee));
    };

    let (launcher_id, prefix) = decode_address(&request.did_id)?;

    if prefix != "did:chia:" {
        return Err(Error::invalid_prefix(&prefix));
    }

    let mut mints = Vec::with_capacity(request.nft_mints.len());
    let mut confirm_info = ConfirmationInfo::default();

    for item in request.nft_mints {
        let royalty_puzzle_hash = item
            .royalty_address
            .map(|address| {
                let (puzzle_hash, prefix) = decode_address(&address)?;
                if prefix != state.network().address_prefix {
                    return Err(Error::invalid_prefix(&prefix));
                }
                Ok(puzzle_hash.into())
            })
            .transpose()?;

        let Some(royalty_ten_thousandths) = item.royalty_percent.to_ten_thousandths() else {
            return Err(Error::invalid_royalty(&item.royalty_percent));
        };

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
            confirm_info.nft_data.insert(data.hash, data);

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
            confirm_info.nft_data.insert(metadata.hash, metadata);

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
            confirm_info.nft_data.insert(data.hash, data);

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

    let (coin_spends, nfts, did) = wallet
        .bulk_mint_nfts(fee, launcher_id.into(), mints, false, true)
        .await?;

    let mut tx = wallet.db.tx().await?;

    for nft in &nfts {
        let info = compute_nft_info(
            None,
            nft.info
                .metadata
                .metadata_hash
                .and_then(|hash| confirm_info.nft_data.get(&hash))
                .map(|data| data.blob.as_ref()),
        );

        tx.insert_nft(NftRow {
            launcher_id: nft.info.launcher_id,
            collection_id: None,
            minter_did: Some(did.info.launcher_id),
            owner_did: nft.info.current_owner,
            sensitive_content: info.sensitive_content,
            name: info.name,
            created_height: None,
            visible: true,
            metadata_hash: nft.info.metadata.metadata_hash,
        })
        .await?;
    }

    tx.commit().await?;

    let summary = summarize(&state, &wallet, coin_spends, confirm_info).await?;

    Ok(BulkMintNftsResponse {
        nft_ids: nfts
            .into_iter()
            .map(|nft| Result::Ok(encode_address(nft.info.launcher_id.into(), "nft")?))
            .collect::<Result<_>>()?,
        summary,
    })
}

#[command]
#[specta]
pub async fn transfer_nft(
    state: State<'_, AppState>,
    nft_id: String,
    address: String,
    fee: Amount,
) -> Result<TransactionSummary> {
    let state = state.lock().await;
    let wallet = state.wallet()?;

    if !state.keychain.has_secret_key(wallet.fingerprint) {
        return Err(Error::no_secret_key());
    }

    let (launcher_id, prefix) = decode_address(&nft_id)?;

    if prefix != "nft" {
        return Err(Error::invalid_prefix(&prefix));
    }

    let (puzzle_hash, prefix) = decode_address(&address)?;
    if prefix != state.network().address_prefix {
        return Err(Error::invalid_prefix(&prefix));
    }

    let Some(fee) = fee.to_mojos(state.unit.decimals) else {
        return Err(Error::invalid_amount(&fee));
    };

    let (coin_spends, _new_nft) = wallet
        .transfer_nft(launcher_id.into(), puzzle_hash.into(), fee, false, true)
        .await?;

    summarize(&state, &wallet, coin_spends, ConfirmationInfo::default()).await
}

#[command]
#[specta]
pub async fn transfer_did(
    state: State<'_, AppState>,
    did_id: String,
    address: String,
    fee: Amount,
) -> Result<TransactionSummary> {
    let state = state.lock().await;
    let wallet = state.wallet()?;

    if !state.keychain.has_secret_key(wallet.fingerprint) {
        return Err(Error::no_secret_key());
    }

    let (launcher_id, prefix) = decode_address(&did_id)?;

    if prefix != "did:chia:" {
        return Err(Error::invalid_prefix(&prefix));
    }

    let (puzzle_hash, prefix) = decode_address(&address)?;
    if prefix != state.network().address_prefix {
        return Err(Error::invalid_prefix(&prefix));
    }

    let Some(fee) = fee.to_mojos(state.unit.decimals) else {
        return Err(Error::invalid_amount(&fee));
    };

    let (coin_spends, _new_ndid) = wallet
        .transfer_did(launcher_id.into(), puzzle_hash.into(), fee, false, true)
        .await?;

    summarize(&state, &wallet, coin_spends, ConfirmationInfo::default()).await
}

#[command]
#[specta]
pub async fn sign_transaction(
    state: State<'_, AppState>,
    coin_spends: Vec<CoinSpendJson>,
) -> Result<SpendBundleJson> {
    let state = state.lock().await;
    let wallet = state.wallet()?;

    let (_mnemonic, Some(master_sk)) = state.keychain.extract_secrets(wallet.fingerprint, b"")?
    else {
        return Err(Error::no_secret_key());
    };

    let spend_bundle = wallet
        .sign_transaction(
            coin_spends.iter().map(rust_spend).collect::<Result<_>>()?,
            &if state.config.network.network_id == "mainnet" {
                AggSigConstants::new(MAINNET_CONSTANTS.agg_sig_me_additional_data)
            } else {
                AggSigConstants::new(TESTNET11_CONSTANTS.agg_sig_me_additional_data)
            },
            master_sk,
        )
        .await?;

    Ok(json_bundle(&spend_bundle))
}

#[command]
#[specta]
pub async fn submit_transaction(
    state: State<'_, AppState>,
    spend_bundle: SpendBundleJson,
) -> Result<()> {
    let state = state.lock().await;
    let wallet = state.wallet()?;

    let spend_bundle = rust_bundle(&spend_bundle)?;

    let mut tx = wallet.db.tx().await?;

    insert_transaction(
        &mut tx,
        spend_bundle.name(),
        Transaction::from_coin_spends(spend_bundle.coin_spends)?,
        spend_bundle.aggregated_signature,
    )
    .await?;

    tx.commit().await?;

    Ok(())
}

#[derive(Default)]
struct ConfirmationInfo {
    did_names: HashMap<Bytes32, String>,
    nft_data: HashMap<Bytes32, Data>,
}

async fn summarize(
    state: &MutexGuard<'_, AppStateInner>,
    wallet: &Wallet,
    coin_spends: Vec<CoinSpend>,
    cache: ConfirmationInfo,
) -> Result<TransactionSummary> {
    let transaction =
        Transaction::from_coin_spends(coin_spends.clone()).map_err(WalletError::Parse)?;

    let mut inputs = Vec::with_capacity(transaction.inputs.len());

    for input in transaction.inputs {
        let coin = input.coin_spend.coin;
        let mut amount = Amount::from_mojos(coin.amount as u128, state.unit.decimals);

        let (kind, p2_puzzle_hash) = match input.kind {
            CoinKind::Unknown => {
                let kind = if wallet.db.is_p2_puzzle_hash(coin.puzzle_hash).await? {
                    InputKind::Xch
                } else {
                    InputKind::Unknown
                };
                (kind, coin.puzzle_hash)
            }
            CoinKind::Launcher => (InputKind::Launcher, coin.puzzle_hash),
            CoinKind::Cat {
                asset_id,
                p2_puzzle_hash,
            } => {
                let cat = wallet.db.cat(asset_id).await?;
                let kind = InputKind::Cat {
                    asset_id: hex::encode(asset_id),
                    name: cat.as_ref().and_then(|cat| cat.name.clone()),
                    ticker: cat.as_ref().and_then(|cat| cat.ticker.clone()),
                    icon_url: cat.as_ref().and_then(|cat| cat.icon_url.clone()),
                };
                amount = Amount::from_mojos(coin.amount as u128, 3);
                (kind, p2_puzzle_hash)
            }
            CoinKind::Did { info } => {
                let name = if let Some(name) = cache.did_names.get(&info.launcher_id).cloned() {
                    Some(name)
                } else {
                    wallet.db.did_name(info.launcher_id).await?
                };

                let kind = InputKind::Did {
                    launcher_id: encode_address(info.launcher_id.into(), "did:chia:")?,
                    name,
                };

                (kind, info.p2_puzzle_hash)
            }
            CoinKind::Nft { info, metadata } => {
                let extracted = extract_nft_data(Some(&wallet.db), metadata, &cache).await?;

                let kind = InputKind::Nft {
                    launcher_id: encode_address(info.launcher_id.into(), "nft")?,
                    image_data: extracted.image_data,
                    image_mime_type: extracted.image_mime_type,
                    name: extracted.name,
                };

                (kind, info.p2_puzzle_hash)
            }
        };

        let address = encode_address(p2_puzzle_hash.into(), &state.network().address_prefix)?;

        let mut outputs = Vec::new();

        for output in input.outputs {
            let amount = match output.kind {
                ChildKind::Cat { .. } => Amount::from_mojos(output.coin.amount as u128, 3),
                _ => Amount::from_mojos(output.coin.amount as u128, state.unit.decimals),
            };

            let p2_puzzle_hash = match output.kind {
                ChildKind::Unknown { hint } => hint.unwrap_or(output.coin.puzzle_hash),
                ChildKind::Launcher => output.coin.puzzle_hash,
                ChildKind::Cat { p2_puzzle_hash, .. } => p2_puzzle_hash,
                ChildKind::Did { info, .. } => info.p2_puzzle_hash,
                ChildKind::Nft { info, .. } => info.p2_puzzle_hash,
            };

            let address = encode_address(p2_puzzle_hash.into(), &state.network().address_prefix)?;

            outputs.push(Output {
                coin_id: hex::encode(output.coin.coin_id()),
                amount,
                address,
                receiving: wallet.db.is_p2_puzzle_hash(p2_puzzle_hash).await?,
            });
        }

        inputs.push(Input {
            coin_id: hex::encode(coin.coin_id()),
            amount,
            address,
            kind,
            outputs,
        });
    }

    Ok(TransactionSummary {
        fee: Amount::from_mojos(transaction.fee as u128, state.unit.decimals),
        inputs,
        coin_spends: coin_spends.iter().map(json_spend).collect(),
    })
}

#[derive(Debug, Default)]
struct ExtractedNftData {
    image_data: Option<String>,
    image_mime_type: Option<String>,
    name: Option<String>,
}

async fn extract_nft_data(
    db: Option<&Database>,
    onchain_metadata: Option<NftMetadata>,
    cache: &ConfirmationInfo,
) -> Result<ExtractedNftData> {
    let mut result = ExtractedNftData::default();

    let Some(onchain_metadata) = onchain_metadata else {
        return Ok(result);
    };

    if let Some(data_hash) = onchain_metadata.data_hash {
        if let Some(data) = cache.nft_data.get(&data_hash) {
            result.image_data = Some(BASE64_STANDARD.encode(&data.blob));
            result.image_mime_type = Some(data.mime_type.clone());
        } else if let Some(db) = &db {
            if let Some(data) = db.fetch_nft_data(data_hash).await? {
                result.image_data = Some(BASE64_STANDARD.encode(&data.blob));
                result.image_mime_type = Some(data.mime_type);
            }
        }
    }

    if let Some(metadata_hash) = onchain_metadata.metadata_hash {
        if let Some(metadata) = cache.nft_data.get(&metadata_hash) {
            let info = compute_nft_info(None, Some(&metadata.blob));
            result.name = info.name;
        } else if let Some(db) = &db {
            if let Some(metadata) = db.fetch_nft_data(metadata_hash).await? {
                let info = compute_nft_info(None, Some(&metadata.blob));
                result.name = info.name;
            }
        }
    }

    Ok(result)
}

fn json_bundle(spend_bundle: &SpendBundle) -> SpendBundleJson {
    SpendBundleJson {
        coin_spends: spend_bundle.coin_spends.iter().map(json_spend).collect(),
        aggregated_signature: format!(
            "0x{}",
            hex::encode(spend_bundle.aggregated_signature.to_bytes())
        ),
    }
}

fn json_spend(coin_spend: &CoinSpend) -> CoinSpendJson {
    CoinSpendJson {
        coin: json_coin(&coin_spend.coin),
        puzzle_reveal: hex::encode(&coin_spend.puzzle_reveal),
        solution: hex::encode(&coin_spend.solution),
    }
}

fn json_coin(coin: &Coin) -> CoinJson {
    CoinJson {
        parent_coin_info: format!("0x{}", hex::encode(coin.parent_coin_info)),
        puzzle_hash: format!("0x{}", hex::encode(coin.puzzle_hash)),
        amount: coin.amount,
    }
}

fn rust_bundle(spend_bundle: &SpendBundleJson) -> Result<SpendBundle> {
    Ok(SpendBundle {
        coin_spends: spend_bundle
            .coin_spends
            .iter()
            .map(rust_spend)
            .collect::<Result<_>>()?,
        aggregated_signature: Signature::from_bytes(&decode_hex_sized(
            &spend_bundle.aggregated_signature,
        )?)?,
    })
}

fn rust_spend(coin_spend: &CoinSpendJson) -> Result<CoinSpend> {
    Ok(CoinSpend {
        coin: rust_coin(&coin_spend.coin)?,
        puzzle_reveal: decode_hex(&coin_spend.puzzle_reveal)?.into(),
        solution: decode_hex(&coin_spend.solution)?.into(),
    })
}

fn rust_coin(coin: &CoinJson) -> Result<Coin> {
    Ok(Coin {
        parent_coin_info: decode_hex_sized(&coin.parent_coin_info)?.into(),
        puzzle_hash: decode_hex_sized(&coin.puzzle_hash)?.into(),
        amount: coin.amount,
    })
}

fn decode_hex(hex: &str) -> Result<Vec<u8>> {
    if let Some(stripped) = hex.strip_prefix("0x") {
        Ok(hex::decode(stripped)?)
    } else {
        Ok(hex::decode(hex)?)
    }
}

fn decode_hex_sized<const N: usize>(hex: &str) -> Result<[u8; N]> {
    let bytes = decode_hex(hex)?;
    Ok(bytes.try_into()?)
}
