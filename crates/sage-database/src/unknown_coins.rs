use chia::protocol::Bytes32;
use sqlx::SqliteExecutor;

use crate::{error::Result, Database, DatabaseTx};

impl Database {
    pub async fn insert_unknown_coin(&self, coin_id: Bytes32) -> Result<()> {
        insert_unknown_coin(&self.pool, coin_id).await
    }
}

impl<'a> DatabaseTx<'a> {
    pub async fn insert_unknown_coin(&mut self, coin_id: Bytes32) -> Result<()> {
        insert_unknown_coin(&mut *self.tx, coin_id).await
    }
}

async fn insert_unknown_coin(conn: impl SqliteExecutor<'_>, coin_id: Bytes32) -> Result<()> {
    let coin_id = coin_id.as_ref();

    sqlx::query!(
        "
        REPLACE INTO `unknown_coins` (`coin_id`) VALUES (?)
        ",
        coin_id
    )
    .execute(conn)
    .await?;

    Ok(())
}
