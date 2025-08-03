use chia::protocol::Bytes32;
use sqlx::query;

use crate::{Asset, CoinRow, DatabaseTx, Result};

#[derive(Debug, Clone, Copy)]
pub struct OptionCoinInfo {
    pub underlying_coin_hash: Bytes32,
    pub underlying_delegated_puzzle_hash: Bytes32,
    pub strike_asset_hash: Bytes32,
    pub strike_amount: u64,
}

#[derive(Debug, Clone)]
pub struct OptionRow {
    pub asset: Asset,
    pub option_info: OptionCoinInfo,
    pub coin_row: CoinRow,
}

impl DatabaseTx<'_> {
    pub async fn insert_option(&mut self, hash: Bytes32, coin_info: &OptionCoinInfo) -> Result<()> {
        let hash = hash.as_ref();
        let underlying_coin_hash = coin_info.underlying_coin_hash.as_ref();
        let underlying_delegated_puzzle_hash = coin_info.underlying_delegated_puzzle_hash.as_ref();
        let strike_asset_hash = coin_info.strike_asset_hash.as_ref();
        let strike_amount = coin_info.strike_amount.to_be_bytes().to_vec();

        query!(
            "
            INSERT OR IGNORE INTO options (
                asset_id, underlying_coin_id, underlying_delegated_puzzle_hash, strike_asset_id, strike_amount
            )
            VALUES (
                (SELECT id FROM assets WHERE hash = ?),
                (SELECT id FROM coins WHERE hash = ?),
                ?,
                (SELECT id FROM assets WHERE hash = ?),
                ?
            )
            ",
            hash,
            underlying_coin_hash,
            underlying_delegated_puzzle_hash,
            strike_asset_hash,
            strike_amount
        )
        .execute(&mut *self.tx)
        .await?;

        Ok(())
    }
}
