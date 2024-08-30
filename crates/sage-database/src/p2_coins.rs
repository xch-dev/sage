use chia::protocol::Bytes32;
use sqlx::SqliteExecutor;

use crate::{error::Result, Database, DatabaseTx};

impl Database {
    pub async fn insert_p2_coin(&self, coin_id: Bytes32) -> Result<()> {
        insert_p2_coin(&self.pool, coin_id).await
    }
}

impl<'a> DatabaseTx<'a> {
    pub async fn insert_p2_coin(&mut self, coin_id: Bytes32) -> Result<()> {
        insert_p2_coin(&mut *self.tx, coin_id).await
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
