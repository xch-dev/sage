use bigdecimal::BigDecimal;
use chia_wallet_sdk::{decode_address, encode_address};
use sage_api::{Amount, CatRecord, CoinRecord, DidRecord, NftRecord, SyncStatus};
use specta::specta;
use tauri::{command, State};

use crate::{app_state::AppState, error::Result};

#[command]
#[specta]
pub async fn get_sync_status(state: State<'_, AppState>) -> Result<SyncStatus> {
    let state = state.lock().await;
    let wallet = state.wallet()?;

    let mut tx = wallet.db.tx().await?;

    let balance = tx.p2_balance().await?;
    let total_coins = tx.total_coin_count().await?;
    let synced_coins = tx.synced_coin_count().await?;

    let max = tx.derivation_index(false).await? - 1;
    let max_used = tx.max_used_derivation_index(false).await?;
    let mut index = max_used.map_or(0, |i| i + 1);
    if index > max {
        index = max;
    }
    let p2_puzzle_hash = tx.p2_puzzle_hash(index, false).await?;

    tx.commit().await?;

    Ok(SyncStatus {
        balance: Amount::from_mojos(balance, state.unit().decimals),
        unit: state.unit().clone(),
        total_coins,
        synced_coins,
        receive_address: encode_address(p2_puzzle_hash.to_bytes(), state.prefix())?,
    })
}

#[command]
#[specta]
pub async fn get_coins(state: State<'_, AppState>) -> Result<Vec<CoinRecord>> {
    let state = state.lock().await;
    let wallet = state.wallet()?;
    let coin_states = wallet.db.p2_coin_states().await?;

    coin_states
        .into_iter()
        .map(|cs| {
            Ok(CoinRecord {
                coin_id: hex::encode(cs.coin.coin_id()),
                address: encode_address(cs.coin.puzzle_hash.to_bytes(), state.prefix())?,
                amount: Amount::from_mojos(cs.coin.amount as u128, state.unit().decimals),
                created_height: cs.created_height,
                spent_height: cs.spent_height,
            })
        })
        .collect()
}

#[command]
#[specta]
pub async fn get_cats(state: State<'_, AppState>) -> Result<Vec<CatRecord>> {
    let state = state.lock().await;
    let wallet = state.wallet()?;
    let cats = wallet.db.cats().await?;

    cats.into_iter()
        .map(|cat| {
            Ok(CatRecord {
                asset_id: hex::encode(cat.asset_id),
                name: cat.name,
                description: cat.description,
                ticker: cat.ticker,
                precision: cat.precision,
                icon_url: cat.icon_url,
            })
        })
        .collect()
}

#[command]
#[specta]
pub async fn get_dids(state: State<'_, AppState>) -> Result<Vec<DidRecord>> {
    let state = state.lock().await;
    let wallet = state.wallet()?;

    wallet
        .db
        .did_coins()
        .await?
        .into_iter()
        .map(|did| {
            Ok(DidRecord {
                encoded_id: encode_address(did.info.launcher_id.to_bytes(), "did:chia:")?,
                launcher_id: hex::encode(did.info.launcher_id),
                coin_id: hex::encode(did.coin.coin_id()),
                address: encode_address(did.info.p2_puzzle_hash.to_bytes(), state.prefix())?,
            })
        })
        .collect()
}

#[command]
#[specta]
pub async fn get_nfts(state: State<'_, AppState>) -> Result<Vec<NftRecord>> {
    let state = state.lock().await;
    let wallet = state.wallet()?;

    wallet
        .db
        .nft_coins()
        .await?
        .into_iter()
        .map(|nft| {
            Ok(NftRecord {
                encoded_id: encode_address(nft.info.launcher_id.to_bytes(), "nft")?,
                launcher_id: hex::encode(nft.info.launcher_id),
                coin_id: hex::encode(nft.coin.coin_id()),
                address: encode_address(nft.info.p2_puzzle_hash.to_bytes(), state.prefix())?,
                royalty_address: encode_address(
                    nft.info.royalty_puzzle_hash.to_bytes(),
                    state.prefix(),
                )?,
                royalty_percent: (BigDecimal::from(nft.info.royalty_ten_thousandths)
                    / BigDecimal::from(100))
                .to_string(),
            })
        })
        .collect()
}

#[command]
#[specta]
pub async fn validate_address(state: State<'_, AppState>, address: String) -> Result<bool> {
    let state = state.lock().await;
    let Some((_puzzle_hash, prefix)) = decode_address(&address).ok() else {
        return Ok(false);
    };
    Ok(prefix == state.prefix())
}
