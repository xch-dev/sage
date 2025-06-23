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
