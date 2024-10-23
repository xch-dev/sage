use chia::{
    protocol::{Bytes32, CoinSpend},
    puzzles::nft::NftMetadata,
};
use chia_wallet_sdk::{
    decode_address, encode_address, AggSigConstants, MAINNET_CONSTANTS, TESTNET11_CONSTANTS,
};
use sage_api::{Amount, BulkMintNfts, BulkMintNftsResponse};
use sage_database::CatRow;
use sage_wallet::{Wallet, WalletNftMint};
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
) -> Result<()> {
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

    transact(&state, &wallet, coin_spends).await?;

    Ok(())
}

#[command]
#[specta]
pub async fn combine(state: State<'_, AppState>, coin_ids: Vec<String>, fee: Amount) -> Result<()> {
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
        .combine_xch(coins, fee, Vec::new(), false, true)
        .await?;

    transact(&state, &wallet, coin_spends).await?;

    Ok(())
}

#[command]
#[specta]
pub async fn split(
    state: State<'_, AppState>,
    coin_ids: Vec<String>,
    output_count: u32,
    fee: Amount,
) -> Result<()> {
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
        .split_xch(&coins, output_count as usize, fee, Vec::new(), false, true)
        .await?;

    transact(&state, &wallet, coin_spends).await?;

    Ok(())
}

#[command]
#[specta]
pub async fn issue_cat(
    state: State<'_, AppState>,
    name: String,
    amount: Amount,
    fee: Amount,
) -> Result<()> {
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

    transact(&state, &wallet, coin_spends).await?;

    wallet
        .db
        .maybe_insert_cat(CatRow {
            asset_id,
            name: Some(name),
            ticker: None,
            description: None,
            icon_url: None,
            visible: true,
        })
        .await?;

    Ok(())
}

#[command]
#[specta]
pub async fn send_cat(
    state: State<'_, AppState>,
    asset_id: String,
    address: String,
    amount: Amount,
    fee: Amount,
) -> Result<()> {
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

    transact(&state, &wallet, coin_spends).await?;

    Ok(())
}

#[command]
#[specta]
pub async fn create_did(state: State<'_, AppState>, name: String, fee: Amount) -> Result<()> {
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
        .insert_new_did(did.info.launcher_id, Some(name), true)
        .await?;

    transact(&state, &wallet, coin_spends).await?;

    Ok(())
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

    for item in request.nft_mints {
        let data_hash = item
            .data_hash
            .map(hex::decode)
            .transpose()?
            .map(TryInto::try_into)
            .transpose()?;

        let metadata_hash = item
            .metadata_hash
            .map(hex::decode)
            .transpose()?
            .map(TryInto::try_into)
            .transpose()?;

        let license_hash = item
            .license_hash
            .map(hex::decode)
            .transpose()?
            .map(TryInto::try_into)
            .transpose()?;

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

    let (coin_spends, nfts, _did) = wallet
        .bulk_mint_nfts(fee, launcher_id.into(), mints, false, true)
        .await?;

    transact(&state, &wallet, coin_spends).await?;

    Ok(BulkMintNftsResponse {
        nft_ids: nfts
            .into_iter()
            .map(|nft| Result::Ok(encode_address(nft.info.launcher_id.into(), "nft")?))
            .collect::<Result<_>>()?,
    })
}

async fn transact(
    state: &MutexGuard<'_, AppStateInner>,
    wallet: &Wallet,
    coin_spends: Vec<CoinSpend>,
) -> Result<()> {
    let (_mnemonic, Some(master_sk)) = state.keychain.extract_secrets(wallet.fingerprint, b"")?
    else {
        return Err(Error::no_secret_key());
    };

    let spend_bundle = wallet
        .sign_transaction(
            coin_spends,
            &if state.config.network.network_id == "mainnet" {
                AggSigConstants::new(MAINNET_CONSTANTS.agg_sig_me_additional_data)
            } else {
                AggSigConstants::new(TESTNET11_CONSTANTS.agg_sig_me_additional_data)
            },
            master_sk,
        )
        .await?;

    let Some(peak) = wallet.db.latest_peak().await? else {
        return Err(Error::no_peak());
    };

    wallet.insert_transaction(&spend_bundle, peak.0).await?;

    Ok(())
}
