use chia_wallet_sdk::prelude::*;
use sqlx::{Row, SqliteExecutor};

use crate::{Asset, AssetKind, Convert, Database, Result};

/// Resolves a coin's p2 puzzle to the wallet's own custody puzzle hash, if there is one.
/// Requires `p2_puzzles` to be joined for the coin, followed by [`CUSTODY_JOINS`].
///
/// Only public key (0), arbor (3), external (4) and P2 singleton (5) puzzles are plain custody.
/// Wrappers such as clawbacks (1) and option underlyings (2) resolve to the receiver or creator,
/// and P2 conditions or singletons (6) to the P2 singleton with the same launcher id and nonce, but
/// only when that's one of our own custody puzzles. Anything else, including counterparty puzzle
/// hashes and p2 puzzle kinds this query doesn't know about, resolves to `NULL` so it's never
/// shown as an address.
pub(crate) const CUSTODY_COLUMNS: &str = "
    CASE
        WHEN p2_puzzles.kind IN (0, 3, 4, 5) THEN p2_puzzles.hash
        WHEN p2_puzzles.kind = 1 THEN clawback_receiver.hash
        WHEN p2_puzzles.kind = 2 THEN option_creator.hash
        WHEN p2_puzzles.kind = 6 THEN conditions_or_singleton_owner.hash
    END AS custody_p2_puzzle_hash";

pub(crate) const CUSTODY_JOINS: &str = "
    LEFT JOIN clawbacks ON clawbacks.p2_puzzle_id = p2_puzzles.id
    LEFT JOIN p2_puzzles AS clawback_receiver
        ON clawback_receiver.hash = clawbacks.receiver_puzzle_hash
        AND clawback_receiver.kind IN (0, 3, 4, 5)
    LEFT JOIN p2_options ON p2_options.p2_puzzle_id = p2_puzzles.id
    LEFT JOIN p2_puzzles AS option_creator
        ON option_creator.hash = p2_options.creator_puzzle_hash
        AND option_creator.kind IN (0, 3, 4, 5)
    LEFT JOIN p2_conditions_or_singletons
        ON p2_conditions_or_singletons.p2_puzzle_id = p2_puzzles.id
    LEFT JOIN p2_singletons AS conditions_or_singleton_p2_singleton
        ON conditions_or_singleton_p2_singleton.launcher_id = p2_conditions_or_singletons.launcher_id
        AND conditions_or_singleton_p2_singleton.nonce = p2_conditions_or_singletons.nonce
    LEFT JOIN p2_puzzles AS conditions_or_singleton_owner
        ON conditions_or_singleton_owner.id = conditions_or_singleton_p2_singleton.p2_puzzle_id";

#[derive(Debug, Clone)]
pub struct Transaction {
    pub height: u32,
    pub timestamp: Option<u64>,
    pub spent: Vec<TransactionCoin>,
    pub created: Vec<TransactionCoin>,
}

#[derive(Debug, Clone)]
pub struct TransactionCoin {
    pub coin: Coin,
    pub asset: Asset,
    /// The wallet's own custody puzzle hash for this coin, which is safe to display as an address.
    /// This is `None` for coins that aren't ours, and is never a wrapper such as a clawback or option.
    pub custody_p2_puzzle_hash: Option<Bytes32>,
}

impl Database {
    pub async fn transaction(&self, height: u32) -> Result<Option<Transaction>> {
        transaction(&self.pool, height).await
    }

    pub async fn transactions(
        &self,
        find_value: Option<String>,
        sort_ascending: bool,
        limit: u32,
        offset: u32,
    ) -> Result<(Vec<Transaction>, u32)> {
        transactions(&self.pool, find_value, sort_ascending, limit, offset).await
    }
}

// Helper function to create a TransactionCoin from a database row
pub(crate) fn create_transaction_coin(row: &sqlx::sqlite::SqliteRow) -> Result<TransactionCoin> {
    let coin = Coin::new(
        row.get::<Vec<u8>, _>("parent_coin_hash").convert()?,
        row.get::<Vec<u8>, _>("puzzle_hash").convert()?,
        row.get::<Vec<u8>, _>("amount").convert()?,
    );

    let asset = Asset {
        hash: row.get::<Vec<u8>, _>("asset_hash").convert()?,
        name: row.get::<Option<String>, _>("asset_name"),
        ticker: row.get::<Option<String>, _>("asset_ticker"),
        precision: row.get::<i64, _>("asset_precision").convert()?,
        icon_url: row.get::<Option<String>, _>("asset_icon_url"),
        description: row.get::<Option<String>, _>("asset_description"),
        is_sensitive_content: row.get::<bool, _>("asset_is_sensitive_content"),
        is_visible: row.get::<bool, _>("asset_is_visible"),
        kind: row
            .get::<Option<i64>, _>("asset_kind")
            .map(Convert::convert)
            .transpose()?
            .unwrap_or(AssetKind::Token),
        hidden_puzzle_hash: row
            .get::<Option<Vec<u8>>, _>("asset_hidden_puzzle_hash")
            .convert()?,
    };

    let custody_p2_puzzle_hash = row
        .get::<Option<Vec<u8>>, _>("custody_p2_puzzle_hash")
        .convert()?;

    Ok(TransactionCoin {
        coin,
        asset,
        custody_p2_puzzle_hash,
    })
}

async fn transaction(conn: impl SqliteExecutor<'_>, height: u32) -> Result<Option<Transaction>> {
    let sql = format!(
        "SELECT
            height, timestamp, coin_id, puzzle_hash, parent_coin_hash, amount,
            is_created_in_block, is_spent_in_block, asset_hash, asset_description,
            asset_is_visible, asset_is_sensitive_content, asset_name, asset_icon_url,
            asset_kind, asset_ticker, asset_precision, asset_hidden_puzzle_hash,
            {CUSTODY_COLUMNS}
        FROM transaction_coins
        LEFT JOIN p2_puzzles ON p2_puzzles.hash = transaction_coins.p2_puzzle_hash
        {CUSTODY_JOINS}
        WHERE height = ?"
    );

    let rows = sqlx::query(&sql).bind(height).fetch_all(conn).await?;

    if rows.is_empty() {
        return Ok(None);
    }

    let mut spent_coins = Vec::new();
    let mut created_coins = Vec::new();
    let mut timestamp = None;

    for row in rows {
        if timestamp.is_none() {
            timestamp = row.get::<Option<i64>, _>("timestamp").map(|ts| ts as u64);
        }

        let transaction_coin = create_transaction_coin(&row)?;

        // these represent whether the coins was spent and/or created in this block
        if row.get::<i64, _>("is_spent_in_block") == 1 {
            spent_coins.push(transaction_coin.clone());
        }

        if row.get::<i64, _>("is_created_in_block") == 1 {
            created_coins.push(transaction_coin);
        }
    }

    Ok(Some(Transaction {
        height,
        timestamp,
        spent: spent_coins,
        created: created_coins,
    }))
}

async fn transactions(
    conn: impl SqliteExecutor<'_>,
    find_value: Option<String>,
    sort_ascending: bool,
    limit: u32,
    offset: u32,
) -> Result<(Vec<Transaction>, u32)> {
    let mut query = sqlx::QueryBuilder::new(format!(
        "SELECT
            height, timestamp, coin_id, puzzle_hash, parent_coin_hash, amount,
            is_created_in_block, is_spent_in_block, asset_hash, asset_description,
            asset_is_visible, asset_is_sensitive_content, asset_name, asset_icon_url,
            asset_kind, asset_ticker, asset_precision, asset_hidden_puzzle_hash,
            {CUSTODY_COLUMNS},
            COUNT(*) OVER() as total_count
        FROM transaction_coins
        LEFT JOIN p2_puzzles ON p2_puzzles.hash = transaction_coins.p2_puzzle_hash
        {CUSTODY_JOINS}
        WHERE 1=1"
    ));

    if let Some(find_value) = find_value {
        query.push(" AND (asset_name LIKE ");
        query.push_bind(format!("%{find_value}%"));
        query.push(" OR asset_ticker LIKE ");
        query.push_bind(format!("%{find_value}%"));

        if is_valid_asset_id(&find_value) {
            query.push(" OR asset_hash = X'");
            query.push(find_value.clone());
            query.push("'");
        }

        // match on nft or did launcher id
        if let Some(puzzle_hash) = puzzle_hash_from_address(&find_value) {
            query.push(" OR asset_hash = X'");
            query.push(puzzle_hash);
            query.push("'");
        }

        // match on height if the find value is parsable as a u32
        if let Ok(height) = find_value.parse::<u32>() {
            query.push(" OR height = ");
            query.push_bind(height);
        }
        query.push(")");
    }

    if sort_ascending {
        query.push(" ORDER BY height ASC");
    } else {
        query.push(" ORDER BY height DESC");
    }

    query.push(" LIMIT ");
    query.push_bind(limit);
    query.push(" OFFSET ");
    query.push_bind(offset);
    let query = query.build();

    let rows = query.fetch_all(conn).await?;
    let total_count = rows
        .first()
        .map_or(Ok(0), |row| row.get::<i64, _>("total_count").try_into())?;

    let transactions = group_rows_into_transactions(rows, sort_ascending)?;

    Ok((transactions, total_count as u32))
}

pub fn is_valid_asset_id(asset_id: &str) -> bool {
    asset_id.len() == 64 && asset_id.chars().all(|c| c.is_ascii_hexdigit())
}

pub fn puzzle_hash_from_address(address: &str) -> Option<String> {
    chia_wallet_sdk::utils::Address::decode(address)
        .map(|decoded| hex::encode(decoded.puzzle_hash.as_ref()))
        .ok()
}

// Helper function to group rows by height and create Transaction structs
fn group_rows_into_transactions(
    rows: Vec<sqlx::sqlite::SqliteRow>,
    sort_ascending: bool,
) -> Result<Vec<Transaction>> {
    use std::collections::HashMap;

    #[allow(clippy::type_complexity)]
    let mut transactions_by_height: HashMap<
        u32,
        (Option<u64>, Vec<TransactionCoin>, Vec<TransactionCoin>),
    > = HashMap::new();

    for row in rows {
        let height: u32 = row.get("height");
        let timestamp: Option<i64> = row.get("timestamp");
        let is_spent_in_block: i64 = row.get("is_spent_in_block");
        let is_created_in_block: i64 = row.get("is_created_in_block");

        let transaction_coin = create_transaction_coin(&row)?;

        let entry = transactions_by_height
            .entry(height)
            .or_insert_with(|| (timestamp.map(|ts| ts as u64), Vec::new(), Vec::new()));

        // these represent whether the coin was spent and/or created in this block
        if is_spent_in_block == 1 {
            entry.1.push(transaction_coin.clone());
        }

        if is_created_in_block == 1 {
            entry.2.push(transaction_coin);
        }
    }

    let mut transactions = Vec::new();
    for (height, (timestamp, spent_coins, created_coins)) in transactions_by_height {
        transactions.push(Transaction {
            height,
            timestamp,
            spent: spent_coins,
            created: created_coins,
        });
    }

    // Sort transactions by height to maintain the order from the SQL query
    if sort_ascending {
        transactions.sort_by_key(|tx| tx.height);
    } else {
        transactions.sort_by_key(|tx| std::cmp::Reverse(tx.height));
    }

    Ok(transactions)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use chia_wallet_sdk::{
        chia::bls::SecretKey,
        driver::{
            Clawback as ClawbackV1, ClawbackV2, OptionType, OptionUnderlying,
            P2ConditionsOrSingleton, P2Singleton,
        },
        types::puzzles::P2DelegatedConditionsArgs,
    };
    use sqlx::{SqlitePool, migrate};

    use super::*;
    use crate::{DatabaseTx, Derivation};

    const HEIGHT: u32 = 1;

    async fn insert_coin(
        tx: &mut DatabaseTx<'_>,
        expected: &mut HashMap<Bytes32, Option<Bytes32>>,
        puzzle_hash: Bytes32,
        asset_hash: Bytes32,
        p2_puzzle_hash: Bytes32,
        custody_p2_puzzle_hash: Option<Bytes32>,
    ) -> anyhow::Result<()> {
        let coin = Coin::new(Bytes32::new([expected.len() as u8; 32]), puzzle_hash, 1);

        tx.insert_coin(CoinState::new(coin, None, Some(HEIGHT)))
            .await?;
        tx.update_coin(coin.coin_id(), asset_hash, p2_puzzle_hash)
            .await?;

        expected.insert(coin.coin_id(), custody_p2_puzzle_hash);

        Ok(())
    }

    fn token(hash: Bytes32, kind: AssetKind) -> Asset {
        Asset {
            hash,
            name: None,
            ticker: None,
            precision: 3,
            icon_url: None,
            description: None,
            is_sensitive_content: false,
            is_visible: true,
            hidden_puzzle_hash: None,
            kind,
        }
    }

    fn custody_by_coin(coins: &[&TransactionCoin]) -> HashMap<Bytes32, Option<Bytes32>> {
        coins
            .iter()
            .map(|coin| (coin.coin.coin_id(), coin.custody_p2_puzzle_hash))
            .collect()
    }

    #[tokio::test]
    async fn test_history_only_shows_own_custody() -> anyhow::Result<()> {
        let pool = SqlitePool::connect("sqlite::memory:").await?;
        migrate!("../../migrations").run(&pool).await?;
        let db = Database::new(pool);

        let own_key = SecretKey::from_seed(&[1; 32]).public_key();
        let arbor_key = SecretKey::from_seed(&[2; 32]).public_key();
        let own = Bytes32::new([1; 32]);
        let arbor: Bytes32 = P2DelegatedConditionsArgs::new(arbor_key)
            .curry_tree_hash()
            .into();
        let external = Bytes32::new([3; 32]);
        let xch = Bytes32::default();
        let cat_asset_id = Bytes32::new([4; 32]);
        let launcher_id = Bytes32::new([5; 32]);

        let mut expected = HashMap::new();

        let mut tx = db.tx().await?;

        tx.insert_height(HEIGHT).await?;
        tx.insert_custody_p2_puzzle(
            own,
            own_key,
            Derivation {
                derivation_index: 0,
                is_hardened: false,
            },
        )
        .await?;
        tx.insert_arbor_p2_puzzle(arbor_key).await?;
        tx.insert_asset(token(cat_asset_id, AssetKind::Token))
            .await?;
        tx.insert_asset(token(launcher_id, AssetKind::Option))
            .await?;

        // Plain custody puzzles are shown as they are
        insert_coin(&mut tx, &mut expected, own, xch, own, Some(own)).await?;
        insert_coin(&mut tx, &mut expected, arbor, xch, arbor, Some(arbor)).await?;

        // CATs show their inner custody puzzle, never the outer CAT puzzle
        let cat_puzzle_hash = Bytes32::new([6; 32]);
        insert_coin(
            &mut tx,
            &mut expected,
            cat_puzzle_hash,
            cat_asset_id,
            own,
            Some(own),
        )
        .await?;

        // Clawbacks only show the receiver when it's us, for both versions
        let outgoing_v2 = ClawbackV2::new(own, external, 100, 1, true);
        let incoming_v2 = ClawbackV2::new(external, own, 100, 1, true);
        let outgoing_v1 = ClawbackV1::new(100, own, external);
        let incoming_v1 = ClawbackV1::new(100, external, own);

        tx.insert_clawback_p2_puzzle(outgoing_v2).await?;
        tx.insert_clawback_p2_puzzle(incoming_v2).await?;
        tx.insert_clawback_v1_p2_puzzle(outgoing_v1).await?;
        tx.insert_clawback_v1_p2_puzzle(incoming_v1).await?;

        for (wrapper, custody) in [
            (outgoing_v2.tree_hash().into(), None),
            (incoming_v2.tree_hash().into(), Some(own)),
            (outgoing_v1.tree_hash().into(), None),
            (incoming_v1.tree_hash().into(), Some(own)),
        ] {
            insert_coin(&mut tx, &mut expected, wrapper, xch, wrapper, custody).await?;
        }

        // A CAT wrapped in a clawback resolves the same way
        insert_coin(
            &mut tx,
            &mut expected,
            Bytes32::new([7; 32]),
            cat_asset_id,
            incoming_v2.tree_hash().into(),
            Some(own),
        )
        .await?;

        // Option underlying coins only show the creator when it's us
        for (creator_puzzle_hash, custody) in [(own, Some(own)), (external, None)] {
            let underlying = OptionUnderlying::new(
                launcher_id,
                creator_puzzle_hash,
                100,
                1,
                OptionType::Xch { amount: 1 },
            );
            let wrapper = underlying.tree_hash().into();

            tx.insert_option_p2_puzzle(underlying).await?;
            insert_coin(&mut tx, &mut expected, wrapper, xch, wrapper, custody).await?;
        }

        // Vault addresses are plain custody, and coins locked to fixed conditions show the vault
        // address with the same launcher id and nonce, but only when it's ours
        let vault_launcher_id = Bytes32::new([10; 32]);
        let p2_singleton = P2Singleton::new(vault_launcher_id, 0);
        let p2_singleton_hash: Bytes32 = p2_singleton.tree_hash().into();
        tx.insert_p2_singleton_p2_puzzle(p2_singleton).await?;
        insert_coin(
            &mut tx,
            &mut expected,
            p2_singleton_hash,
            xch,
            p2_singleton_hash,
            Some(p2_singleton_hash),
        )
        .await?;

        for (nonce, custody) in [(0, Some(p2_singleton_hash)), (1, None)] {
            let p2 = P2ConditionsOrSingleton::new(vault_launcher_id, nonce, Bytes32::new([11; 32]));
            let wrapper = p2.tree_hash().into();

            tx.insert_p2_conditions_or_singleton_p2_puzzle(p2, None)
                .await?;
            insert_coin(&mut tx, &mut expected, wrapper, xch, wrapper, custody).await?;
        }

        // Coins whose p2 puzzle we don't know show nothing
        insert_coin(&mut tx, &mut expected, external, xch, external, None).await?;

        tx.commit().await?;

        // A p2 puzzle kind added in the future must not break the history or be shown
        let unknown = Bytes32::new([8; 32]);
        sqlx::query("INSERT INTO p2_puzzles (hash, kind) VALUES (?, 99)")
            .bind(unknown.as_ref())
            .execute(&db.pool)
            .await?;

        let mut tx = db.tx().await?;
        insert_coin(&mut tx, &mut expected, unknown, xch, unknown, None).await?;

        // Every coin is also an output of a pending transaction
        let mempool_item_id = Bytes32::new([9; 32]);
        tx.insert_mempool_item(mempool_item_id, Signature::default(), 0)
            .await?;

        for coin_id in expected.keys() {
            tx.insert_mempool_coin(mempool_item_id, *coin_id, false, true)
                .await?;
        }

        tx.commit().await?;

        let (transactions, total) = db.transactions(None, false, 100, 0).await?;
        assert_eq!(total as usize, expected.len());
        assert_eq!(transactions.len(), 1);
        assert_eq!(
            custody_by_coin(&transactions[0].created.iter().collect::<Vec<_>>()),
            expected
        );

        let transaction = db.transaction(HEIGHT).await?.expect("missing transaction");
        assert_eq!(
            custody_by_coin(&transaction.created.iter().collect::<Vec<_>>()),
            expected
        );

        let mempool_items = db.mempool_items_with_coins().await?;
        assert_eq!(mempool_items.len(), 1);
        assert_eq!(
            custody_by_coin(&mempool_items[0].created.iter().collect::<Vec<_>>()),
            expected
        );

        Ok(())
    }
}
