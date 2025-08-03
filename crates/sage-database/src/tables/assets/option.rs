use chia::protocol::Bytes32;
use chia_wallet_sdk::driver::{OptionType, OptionUnderlying};
use sqlx::query;

use crate::{Asset, CoinRow, Convert, Database, DatabaseTx, Result};

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

impl Database {
    pub async fn option_underlying(
        &self,
        launcher_id: Bytes32,
    ) -> Result<Option<OptionUnderlying>> {
        let launcher_id_ref = launcher_id.as_ref();

        let Some(row) = query!(
            "
            SELECT
                creator_puzzle_hash, expiration_seconds,
                (
                    SELECT amount FROM coins
                    WHERE coins.p2_puzzle_id = p2_options.p2_puzzle_id LIMIT 1
                ) AS underlying_amount,
                (SELECT hash FROM assets WHERE id = strike_asset_id) AS strike_asset_hash,
                strike_amount, strike_assets.hidden_puzzle_hash AS strike_hidden_puzzle_hash
            FROM p2_options
            INNER JOIN options ON options.asset_id = p2_options.option_asset_id
            INNER JOIN assets AS strike_assets ON strike_assets.id = options.strike_asset_id
            WHERE option_asset_id = (SELECT id FROM assets WHERE hash = ?)
            ",
            launcher_id_ref
        )
        .fetch_optional(&self.pool)
        .await?
        else {
            return Ok(None);
        };

        let asset_hash: Bytes32 = row.strike_asset_hash.convert()?;
        let amount: u64 = row.strike_amount.convert()?;
        let hidden_puzzle_hash: Option<Bytes32> = row.strike_hidden_puzzle_hash.convert()?;

        Ok(Some(OptionUnderlying::new(
            launcher_id,
            row.creator_puzzle_hash.convert()?,
            row.expiration_seconds.convert()?,
            row.underlying_amount.convert()?,
            if asset_hash == Bytes32::default() {
                OptionType::Xch { amount }
            } else if let Some(hidden_puzzle_hash) = hidden_puzzle_hash {
                OptionType::RevocableCat {
                    asset_id: asset_hash,
                    hidden_puzzle_hash,
                    amount,
                }
            } else {
                OptionType::Cat {
                    asset_id: asset_hash,
                    amount,
                }
            },
        )))
    }
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
