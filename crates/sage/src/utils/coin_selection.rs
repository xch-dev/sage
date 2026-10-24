use std::collections::HashSet;

use chia_wallet_sdk::prelude::*;
use sage_wallet::Wallet;

use crate::{Error, Result, parse_coin_ids};

/// The coins a caller chose to spend first (`coin_ids` on sends and cancels).
///
/// Each one must be a coin the wallet would select by itself: its own, confirmed, unspent,
/// not in the mempool and not locked in an open offer. It must also be a kind the
/// transaction spends: XCH, or a coin of `cat` when the transaction moves that CAT. Anything
/// else is refused rather than spent, so a caller can choose which coins pay, but cannot pull
/// a coin backing an open offer, an NFT or a DID into a send or a fee.
pub async fn selectable_coin_ids(
    wallet: &Wallet,
    coin_ids: Option<Vec<String>>,
    cat: Option<Bytes32>,
) -> Result<Vec<Bytes32>> {
    let requested = parse_coin_ids(coin_ids.unwrap_or_default())?;

    if requested.is_empty() {
        return Ok(Vec::new());
    }

    let mut allowed: HashSet<Bytes32> = wallet
        .db
        .selectable_xch_coins()
        .await?
        .iter()
        .map(Coin::coin_id)
        .collect();

    if let Some(asset_id) = cat {
        allowed.extend(
            wallet
                .db
                .selectable_cat_coins(asset_id)
                .await?
                .iter()
                .map(|cat| cat.coin.coin_id()),
        );
    }

    let mut selected = Vec::with_capacity(requested.len());

    for coin_id in requested {
        if !allowed.contains(&coin_id) {
            return Err(Error::CoinNotSelectable(coin_id));
        }

        if !selected.contains(&coin_id) {
            selected.push(coin_id);
        }
    }

    Ok(selected)
}
