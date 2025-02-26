use sqlx::SqliteExecutor;

use crate::{Database, DatabaseTx, Result};
//use std::error::Error;

impl Database {
    //find null created_unixtime in coin_states
    pub async fn find_created_timestamp_null(&self) -> Result<Option<i64>> {
        find_created_timestamp_null(&self.pool).await
    }
    //find null created_unixtime in coin_states
    pub async fn find_spent_timestamp_null(&self) -> Result<Option<i64>> {
        find_spent_timestamp_null(&self.pool).await
    }
    //from blockinfo get unix_time based on height
    pub async fn get_timestamp_blockinfo(&self, height: u32) -> Result<i64> {
        get_timestamp_blockinfo(&self.pool, height).await
    }
    //insert created timestamp into coin_states
    pub async fn insert_created_timestamp(&self, height: u32, timestamp: u32) -> Result<u32> {
        insert_created_timestamp(&self.pool, height, timestamp).await
    }
    //insert spent timestamp into coin_states
    pub async fn insert_spent_timestamp(&self, height: u32, timestamp: u32) -> Result<u32> {
        insert_spent_timestamp(&self.pool, height, timestamp).await
    }
    //insert timestamp and height into blockinfo
    pub async fn insert_timestamp_height(&self, height: u32, timestamp: u32) -> Result<()> {
        insert_timestamp_height(&self.pool, height, timestamp).await
    }
}

impl<'a> DatabaseTx<'a> {
    //find created_unixtime null in coin_states
    pub async fn find_created_timestamp_null(&mut self) -> Result<Option<i64>> {
        find_created_timestamp_null(&mut *self.tx).await
    }
    //find spent_unixtime null in coin_states
    pub async fn find_spent_timestamp_null(&mut self) -> Result<Option<i64>> {
        find_spent_timestamp_null(&mut *self.tx).await
    }
    //find spent_unixtime null in coin_states
    //pub async fn get_timestamp_blockinfo(&mut self, height: u32) -> Result<i64> {
    //    get_timestamp_blockinfo(&mut *self.tx, height).await
    //}
}

async fn find_created_timestamp_null(conn: impl SqliteExecutor<'_>) -> Result<Option<i64>> {
    let row = sqlx::query!(
        //start with most recent block height since these should be most interesting to end user
        "
            SELECT `created_height`
            FROM `coin_states`
            WHERE `created_unixtime` IS NULL
            ORDER BY `created_height` DESC 
            LIMIT 1;

        ",
    )
    .fetch_optional(conn)
    .await?;

    // Use and_then to handle Option<Option<i64>> and return Option<i64>
    Ok(row.and_then(|r| r.created_height))
}

async fn find_spent_timestamp_null(conn: impl SqliteExecutor<'_>) -> Result<Option<i64>> {
    let row = sqlx::query!(
        //start with most recent block height since these should be most interesting to end user
        "
            SELECT `spent_height`
            FROM `coin_states`
            WHERE `spent_unixtime` IS NULL
            AND `spent_height` IS NOT NULL
            ORDER BY `spent_height` DESC 
            LIMIT 1;            
        ",
    )
    .fetch_optional(conn)
    .await?;

    Ok(row.and_then(|r| r.spent_height))
}

//from blockinfo get unix_time based on height fix me gneale 20250223
async fn get_timestamp_blockinfo(conn: impl SqliteExecutor<'_>, height: u32) -> Result<i64> {
    let row = sqlx::query!(
        "
            SELECT `unix_time`
            FROM `blockinfo`
            WHERE `height` = ?
        ",
        height
    )
    .fetch_one(conn)
    .await?;

    Ok(row.unix_time)
}

//update created timestamp into coin_states
async fn insert_created_timestamp(
    conn: impl SqliteExecutor<'_>,
    height: u32,
    timestamp: u32,
) -> Result<u32> {
    sqlx::query!(
        "
        UPDATE OR IGNORE `coin_states`
        SET `created_unixtime` = ?
        WHERE `created_height` = ?
        ",
        timestamp,
        height
    )
    .execute(conn) // Execute the query
    .await?; // Await the execution to complete

    Ok(timestamp)
}

//update spent timestamp into coin_states
async fn insert_spent_timestamp(
    conn: impl SqliteExecutor<'_>,
    height: u32,
    timestamp: u32,
) -> Result<u32> {
    sqlx::query!(
        "
        UPDATE OR IGNORE `coin_states`
        SET `spent_unixtime` = ?
        WHERE `spent_height` = ?
        ",
        timestamp,
        height
    )
    .execute(conn) // Execute the query
    .await?; // Await the execution to complete

    // Return the timestamp after the update attempt
    Ok(timestamp)
}

//
async fn insert_timestamp_height(
    conn: impl SqliteExecutor<'_>,
    height: u32,
    unix_timestamp: u32,
) -> Result<()> {
    sqlx::query!(
        "
        INSERT OR IGNORE INTO `blockinfo` (
            `height`,
            `unix_time`
        )
        VALUES (?, ?)
        ",
        height,
        unix_timestamp
    )
    .execute(conn)
    .await?;

    Ok(())
}
