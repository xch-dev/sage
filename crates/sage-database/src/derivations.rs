// use chia::{bls::PublicKey, protocol::Bytes32};
// use sqlx::{Row, SqliteExecutor};

// use crate::{
//     into_row, to_bytes, to_bytes32, Database, DatabaseTx, DerivationRow, DerivationSql, Result,
// };

// #[derive(Debug, Clone, Copy)]
// pub struct SyntheticKeyInfo {
//     pub index: u32,
//     pub hardened: bool,
// }

// impl Database {
//     pub async fn derivations(
//         &self,
//         hardened: bool,
//         limit: u32,
//         offset: u32,
//     ) -> Result<(Vec<DerivationRow>, u32)> {
//         derivations(&self.pool, hardened, limit, offset).await
//     }

//     pub async fn synthetic_key(&self, p2_puzzle_hash: Bytes32) -> Result<PublicKey> {
//         synthetic_key(&self.pool, p2_puzzle_hash).await
//     }

//     pub async fn synthetic_key_info(
//         &self,
//         synthetic_key: PublicKey,
//     ) -> Result<Option<SyntheticKeyInfo>> {
//         synthetic_key_info(&self.pool, synthetic_key).await
//     }
// }

// impl DatabaseTx<'_> {
//     pub async fn insert_derivation(
//         &mut self,
//         p2_puzzle_hash: Bytes32,
//         index: u32,
//         hardened: bool,
//         synthetic_key: PublicKey,
//     ) -> Result<()> {
//         insert_derivation(
//             &mut *self.tx,
//             p2_puzzle_hash,
//             index,
//             hardened,
//             synthetic_key,
//         )
//         .await
//     }

//     pub async fn synthetic_key(&mut self, p2_puzzle_hash: Bytes32) -> Result<PublicKey> {
//         synthetic_key(&mut *self.tx, p2_puzzle_hash).await
//     }

//     pub async fn p2_puzzle_hash(&mut self, index: u32, hardened: bool) -> Result<Bytes32> {
//         p2_puzzle_hash(&mut *self.tx, index, hardened).await
//     }
// }

// async fn insert_derivation(
//     conn: impl SqliteExecutor<'_>,
//     p2_puzzle_hash: Bytes32,
//     index: u32,
//     hardened: bool,
//     synthetic_key: PublicKey,
// ) -> Result<()> {
//     let p2_puzzle_hash = p2_puzzle_hash.as_ref();
//     let synthetic_key = synthetic_key.to_bytes();
//     let synthetic_key_ref = synthetic_key.as_ref();
//     sqlx::query!(
//         "
//         INSERT OR IGNORE INTO `derivations` (`p2_puzzle_hash`, `index`, `hardened`, `synthetic_key`)
//         VALUES (?, ?, ?, ?)
//         ",
//         p2_puzzle_hash,
//         index,
//         hardened,
//         synthetic_key_ref
//     )
//     .execute(conn)
//     .await?;
//     Ok(())
// }
// async fn derivations(
//     conn: impl SqliteExecutor<'_>,
//     hardened: bool,
//     limit: u32,
//     offset: u32,
// ) -> Result<(Vec<DerivationRow>, u32)> {
//     let mut query = sqlx::QueryBuilder::new(
//         "
//         SELECT
//             `p2_puzzle_hash`,
//             `index`,
//             `hardened`,
//             `synthetic_key`,
//             COUNT(*) OVER() as total_count
//         FROM `derivations`
//         WHERE `hardened` =
//         ",
//     );

//     query.push_bind(hardened);
//     query.push(" ORDER BY `index` ASC");
//     query.push(" LIMIT ");
//     query.push_bind(limit);
//     query.push(" OFFSET ");
//     query.push_bind(offset);

//     // Build the query and bind the hardened parameter
//     let sql = query.build();
//     let rows = sql.bind(hardened).fetch_all(conn).await?;

//     let Some(first_row) = rows.first() else {
//         return Ok((vec![], 0));
//     };

//     let total: u32 = first_row.try_get("total_count")?;

//     let mut derivations = Vec::with_capacity(rows.len());

//     for row in rows {
//         let sql = DerivationSql {
//             p2_puzzle_hash: row.try_get("p2_puzzle_hash")?,
//             index: row.try_get("index")?,
//             hardened: row.try_get("hardened")?,
//             synthetic_key: row.try_get("synthetic_key")?,
//         };
//         derivations.push(into_row(sql)?);
//     }

//     Ok((derivations, total))
// }

// async fn synthetic_key(
//     conn: impl SqliteExecutor<'_>,
//     p2_puzzle_hash: Bytes32,
// ) -> Result<PublicKey> {
//     let p2_puzzle_hash = p2_puzzle_hash.as_ref();
//     let row = sqlx::query!(
//         "
//         SELECT `synthetic_key`
//         FROM `derivations`
//         WHERE `p2_puzzle_hash` = ?
//         ",
//         p2_puzzle_hash
//     )
//     .fetch_one(conn)
//     .await?;
//     let bytes = row.synthetic_key.as_slice();
//     Ok(PublicKey::from_bytes(&to_bytes(bytes)?)?)
// }

// async fn synthetic_key_info(
//     conn: impl SqliteExecutor<'_>,
//     synthetic_key: PublicKey,
// ) -> Result<Option<SyntheticKeyInfo>> {
//     let synthetic_key = synthetic_key.to_bytes();
//     let synthetic_key_ref = synthetic_key.as_ref();

//     sqlx::query!(
//         "
//         SELECT `index`, `hardened`
//         FROM `derivations`
//         WHERE `synthetic_key` = ?
//         ",
//         synthetic_key_ref
//     )
//     .fetch_optional(conn)
//     .await?
//     .map(|row| {
//         Ok(SyntheticKeyInfo {
//             index: row.index.try_into()?,
//             hardened: row.hardened,
//         })
//     })
//     .transpose()
// }

// async fn p2_puzzle_hash(
//     conn: impl SqliteExecutor<'_>,
//     index: u32,
//     hardened: bool,
// ) -> Result<Bytes32> {
//     let row = sqlx::query!(
//         "
//         SELECT `p2_puzzle_hash`
//         FROM `derivations`
//         WHERE `index` = ?
//         AND `hardened` = ?
//         ",
//         index,
//         hardened
//     )
//     .fetch_one(conn)
//     .await?;
//     to_bytes32(&row.p2_puzzle_hash)
// }
