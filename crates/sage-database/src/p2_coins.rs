use chia::protocol::Bytes32;
use sqlx::SqliteExecutor;

use crate::{error::Result, to_bytes, Database, DatabaseTx};

impl Database {
    pub async fn insert_p2_coin(&self, coin_id: Bytes32) -> Result<()> {
        insert_p2_coin(&self.pool, coin_id).await
    }

    pub async fn p2_balance(&self) -> Result<u128> {
        p2_balance(&self.pool).await
    }
}

impl<'a> DatabaseTx<'a> {
    pub async fn insert_p2_coin(&mut self, coin_id: Bytes32) -> Result<()> {
        insert_p2_coin(&mut *self.tx, coin_id).await
    }

    pub async fn p2_balance(&mut self) -> Result<u128> {
        p2_balance(&mut *self.tx).await
    }
}

async fn insert_p2_coin(conn: impl SqliteExecutor<'_>, coin_id: Bytes32) -> Result<()> {
    let coin_id = coin_id.as_ref();

    sqlx::query!(
        "
        INSERT INTO `p2_coins` (`coin_id`) VALUES (?)
        ",
        coin_id
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn p2_balance(conn: impl SqliteExecutor<'_>) -> Result<u128> {
    let row = sqlx::query!(
        "
        SELECT `amount` FROM `coin_states`
        INNER JOIN `p2_coins` ON `coin_states`.`coin_id` = `p2_coins`.`coin_id`
        WHERE `coin_states`.`spent_height` IS NULL
        "
    )
    .fetch_all(conn)
    .await?;

    row.iter()
        .map(|row| Ok(u64::from_be_bytes(to_bytes(&row.amount)?) as u128))
        .sum::<Result<u128>>()
}
