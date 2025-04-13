use chia::protocol::{Bytes32, Coin};
use chia_wallet_sdk::driver::Cat;
use hex_literal::hex;
use sage_wallet::Wallet;

use crate::{Error, Result};

use super::parse_coin_id;

pub const BURN_PUZZLE_HASH: [u8; 32] =
    hex!("000000000000000000000000000000000000000000000000000000000000dead");

pub async fn fetch_coins(wallet: &Wallet, coin_ids: Vec<String>) -> Result<Vec<Coin>> {
    let coin_ids = coin_ids
        .into_iter()
        .map(parse_coin_id)
        .collect::<Result<Vec<Bytes32>>>()?;

    let mut coins = Vec::new();

    for coin_id in coin_ids {
        let Some(coin_state) = wallet.db.coin_state(coin_id).await? else {
            return Err(Error::MissingCoin(coin_id));
        };

        if coin_state.spent_height.is_some() {
            return Err(Error::CoinSpent(coin_id));
        }

        coins.push(coin_state.coin);
    }

    Ok(coins)
}

pub async fn fetch_cats(wallet: &Wallet, coin_ids: Vec<String>) -> Result<Vec<Cat>> {
    let coin_ids = coin_ids
        .into_iter()
        .map(parse_coin_id)
        .collect::<Result<Vec<Bytes32>>>()?;

    let mut cats = Vec::new();

    for coin_id in coin_ids {
        let Some(coin_state) = wallet.db.coin_state(coin_id).await? else {
            return Err(Error::MissingCoin(coin_id));
        };

        if coin_state.spent_height.is_some() {
            return Err(Error::CoinSpent(coin_id));
        }

        let Some(cat) = wallet.db.cat_coin(coin_id).await? else {
            return Err(Error::MissingCatCoin(coin_id));
        };

        cats.push(cat);
    }

    Ok(cats)
}
