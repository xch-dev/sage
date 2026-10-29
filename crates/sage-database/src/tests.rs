use chia_wallet_sdk::{
    chia::puzzle_types::nft::NftMetadata,
    driver::{P2ConditionsOrSingleton, P2ConditionsOrSingletonVersion, P2Singleton, VaultVersion},
    prelude::*,
    puzzles::NFT_METADATA_UPDATER_DEFAULT_HASH,
    test::{BlsPair, K1Pair, R1Pair},
    types::puzzles::{ANY_METADATA_UPDATER_HASH, HandleNftMetadata},
};
use sqlx::{SqlitePool, migrate, raw_sql, sqlite::SqlitePoolOptions};

use crate::{
    AssetKind, CoinKind, Database, NftUpdaterKind, P2Puzzle, VaultConfig, VaultConfigRow,
    VaultConfigSource, VaultLayout, VaultMember, VaultRecoveryConfig, VaultSigners, VaultState,
};

fn encode<T: ToClvm<Allocator>>(value: &T) -> Vec<u8> {
    let mut allocator = Allocator::new();
    let ptr = value.to_clvm(&mut allocator).unwrap();
    Program::from_clvm(&allocator, ptr).unwrap().to_vec()
}

async fn insert_legacy_nft(
    pool: &SqlitePool,
    launcher_id: Bytes32,
    metadata_updater_puzzle_hash: Bytes32,
    metadata: Vec<u8>,
) -> anyhow::Result<()> {
    let launcher_id = launcher_id.to_vec();
    let metadata_updater_puzzle_hash = metadata_updater_puzzle_hash.to_vec();

    sqlx::query("INSERT INTO assets (hash, kind, precision, is_visible) VALUES (?, 1, 1, TRUE)")
        .bind(&launcher_id)
        .execute(pool)
        .await?;

    // Before the backfill, every NFT was parsed as CHIP-7, which defaults the edition to 1 of 1.
    sqlx::query(
        "
        INSERT INTO nfts (
            asset_id, metadata, metadata_updater_puzzle_hash, royalty_puzzle_hash,
            royalty_basis_points, edition_number, edition_total
        )
        VALUES ((SELECT id FROM assets WHERE hash = ?), ?, ?, ?, 0, 1, 1)
        ",
    )
    .bind(&launcher_id)
    .bind(metadata)
    .bind(metadata_updater_puzzle_hash)
    .bind(vec![0u8; 32])
    .execute(pool)
    .await?;

    Ok(())
}

async fn nft_columns(
    pool: &SqlitePool,
    launcher_id: Bytes32,
) -> anyhow::Result<(i64, Option<String>, Option<String>, Option<i64>)> {
    Ok(sqlx::query_as(
        "
        SELECT metadata_updater_kind, display_name, name, edition_number FROM nfts
        INNER JOIN assets ON assets.id = nfts.asset_id
        WHERE hash = ?
        ",
    )
    .bind(launcher_id.to_vec())
    .fetch_one(pool)
    .await?)
}

#[tokio::test]
async fn backfills_nft_metadata_updater_kinds() -> anyhow::Result<()> {
    let pool = SqlitePool::connect("sqlite::memory:").await?;
    let migrator = migrate!("../../migrations");

    for migration in migrator.iter().filter(|m| m.version < 7) {
        raw_sql(&migration.sql).execute(&pool).await?;
    }

    let default_nft = Bytes32::new([1; 32]);
    let any_nft = Bytes32::new([2; 32]);
    let malformed_any_nft = Bytes32::new([3; 32]);
    let unknown_nft = Bytes32::new([4; 32]);

    let handle_metadata = encode(&HandleNftMetadata {
        display_name: Some("@alice".to_string()),
        image_uris: vec!["https://example.com/alice.png".to_string()],
        image_hash: Some(Bytes32::new([9; 32])),
        ..Default::default()
    });

    insert_legacy_nft(
        &pool,
        default_nft,
        NFT_METADATA_UPDATER_DEFAULT_HASH.into(),
        encode(&NftMetadata::default()),
    )
    .await?;
    insert_legacy_nft(
        &pool,
        any_nft,
        ANY_METADATA_UPDATER_HASH.into(),
        handle_metadata.clone(),
    )
    .await?;
    insert_legacy_nft(
        &pool,
        malformed_any_nft,
        ANY_METADATA_UPDATER_HASH.into(),
        encode(&vec![("dn", vec![1u8, 2, 3])]),
    )
    .await?;
    insert_legacy_nft(&pool, unknown_nft, Bytes32::new([5; 32]), handle_metadata).await?;

    for migration in migrator.iter().filter(|m| m.version >= 7) {
        raw_sql(&migration.sql).execute(&pool).await?;
    }

    let db = Database::new(pool.clone());
    db.run_rust_migrations("XCH".to_string()).await?;

    assert_eq!(
        nft_columns(&pool, default_nft).await?,
        (NftUpdaterKind::Default as i64, None, None, Some(1))
    );
    assert_eq!(
        nft_columns(&pool, any_nft).await?,
        (
            NftUpdaterKind::Any as i64,
            Some("@alice".to_string()),
            Some("@alice".to_string()),
            None
        )
    );
    assert_eq!(
        nft_columns(&pool, malformed_any_nft).await?,
        (NftUpdaterKind::Any as i64, None, None, Some(1))
    );
    // Unknown updaters keep CHIP-7 parsing and never get a display name or name from it.
    assert_eq!(
        nft_columns(&pool, unknown_nft).await?,
        (NftUpdaterKind::Unknown as i64, None, None, Some(1))
    );

    Ok(())
}

async fn migrated_db() -> anyhow::Result<Database> {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await?;
    migrate!("../../migrations").run(&pool).await?;
    Ok(Database::new(pool))
}

#[tokio::test]
async fn vaults_migration_keeps_existing_coins() -> anyhow::Result<()> {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await?;
    let migrator = migrate!("../../migrations");

    for migration in migrator.iter().filter(|m| m.version < 11) {
        raw_sql(&migration.sql).execute(&pool).await?;
    }

    let p2_puzzle_hash = Bytes32::new([1; 32]);
    let coin = Coin::new(Bytes32::new([2; 32]), p2_puzzle_hash, 1000);

    sqlx::query("INSERT INTO p2_puzzles (hash, kind, is_synced) VALUES (?, 0, TRUE)")
        .bind(p2_puzzle_hash.to_vec())
        .execute(&pool)
        .await?;
    sqlx::query("INSERT INTO blocks (height, is_peak) VALUES (5, TRUE)")
        .execute(&pool)
        .await?;
    sqlx::query(
        "
        INSERT INTO coins (hash, asset_id, parent_coin_hash, puzzle_hash, amount, p2_puzzle_id, created_height)
        VALUES (?, 0, ?, ?, ?, (SELECT id FROM p2_puzzles WHERE hash = ?), 5)
        ",
    )
    .bind(coin.coin_id().to_vec())
    .bind(coin.parent_coin_info.to_vec())
    .bind(coin.puzzle_hash.to_vec())
    .bind(coin.amount.to_be_bytes().to_vec())
    .bind(p2_puzzle_hash.to_vec())
    .execute(&pool)
    .await?;

    for migration in migrator.iter().filter(|m| m.version == 11) {
        raw_sql(&migration.sql).execute(&pool).await?;
    }

    let db = Database::new(pool);

    assert_eq!(db.selectable_xch_coins().await?, [coin]);
    assert_eq!(db.xch_balance().await?, 1000);
    assert_eq!(db.custody_p2_puzzle_hashes().await?, [p2_puzzle_hash]);
    assert!(db.unsynced_custody_p2_puzzle_hashes().await?.is_empty());

    Ok(())
}

#[tokio::test]
async fn stores_vault_configs() -> anyhow::Result<()> {
    let db = migrated_db().await?;

    let config = VaultConfig {
        custody: VaultSigners::new(
            vec![
                VaultMember::secp256r1(R1Pair::new(1).pk),
                VaultMember::passkey(R1Pair::new(2).pk),
                VaultMember::secp256k1(K1Pair::new(3).pk),
                VaultMember::Singleton {
                    launcher_id: Bytes32::new([4; 32]),
                    fast_forward: true,
                },
            ],
            3,
        ),
        recovery: VaultSigners::new(
            vec![
                VaultMember::bls(BlsPair::new(5).pk),
                VaultMember::Custom {
                    puzzle_hash: Bytes32::new([6; 32]),
                },
            ],
            1,
        ),
        recovery_timelock: 86_400,
        version: VaultVersion::ForceSingletonRecreation,
    };

    let custody = VaultConfigRow {
        custody_hash: Bytes32::new([7; 32]),
        layout: VaultLayout::Custody(config.clone()),
        source: VaultConfigSource::Memo,
    };

    let recovery = VaultConfigRow {
        custody_hash: Bytes32::new([8; 32]),
        layout: VaultLayout::Recovery(VaultRecoveryConfig {
            custody: config.custody.clone(),
            recovery_timelock: 86_400,
            version: VaultVersion::Legacy,
            recovery_custody_hash: Bytes32::new([7; 32]),
            amount: 1,
        }),
        source: VaultConfigSource::ConfigFile,
    };

    let mut tx = db.tx().await?;
    tx.insert_vault_config(&custody).await?;
    tx.insert_vault_config(&recovery).await?;
    tx.commit().await?;

    assert_eq!(
        db.vault_config(custody.custody_hash).await?,
        Some(custody.clone())
    );
    assert_eq!(
        db.vault_config(recovery.custody_hash).await?,
        Some(recovery)
    );
    assert_eq!(db.vault_config(Bytes32::default()).await?, None);

    // Reinserting a config replaces its members.
    let replaced = VaultConfigRow {
        layout: VaultLayout::Custody(VaultConfig {
            recovery: VaultSigners::new(vec![VaultMember::bls(BlsPair::new(9).pk)], 1),
            ..config
        }),
        ..custody
    };
    let mut tx = db.tx().await?;
    tx.insert_vault_config(&replaced).await?;
    tx.commit().await?;

    assert_eq!(
        db.vault_config(replaced.custody_hash).await?,
        Some(replaced)
    );

    Ok(())
}

/// Inserts a vault coin, along with the height and timestamp of the block it was created in.
async fn insert_vault_coin(
    db: &Database,
    launcher_id: Bytes32,
    coin: Coin,
    custody_hash: Option<Bytes32>,
    created_height: Option<u32>,
) -> anyhow::Result<()> {
    if let Some(height) = created_height {
        db.insert_block(
            height,
            Bytes32::default(),
            Some(i64::from(height) * 100),
            false,
        )
        .await?;
    }

    let mut tx = db.tx().await?;
    tx.insert_coin(CoinState::new(coin, None, created_height))
        .await?;
    tx.update_coin(coin.coin_id(), launcher_id, Bytes32::default())
        .await?;
    tx.insert_vault_coin(coin.coin_id(), custody_hash).await?;
    tx.commit().await?;

    Ok(())
}

async fn set_coin_state(
    db: &Database,
    coin: Coin,
    created_height: Option<u32>,
    spent_height: Option<u32>,
) -> anyhow::Result<()> {
    let mut tx = db.tx().await?;
    if let Some(height) = spent_height {
        tx.insert_height(height).await?;
    }
    tx.insert_coin(CoinState::new(coin, spent_height, created_height))
        .await?;
    tx.commit().await?;
    Ok(())
}

#[tokio::test]
async fn vault_state_follows_lineage() -> anyhow::Result<()> {
    let db = migrated_db().await?;
    let launcher_id = Bytes32::new([1; 32]);
    let custody_hash = Bytes32::new([2; 32]);
    let recovery_hash = Bytes32::new([3; 32]);

    assert_eq!(db.vault_status(launcher_id).await?, None);

    let mut tx = db.tx().await?;
    tx.insert_vault(launcher_id).await?;
    tx.insert_vault_config(&VaultConfigRow {
        custody_hash: recovery_hash,
        layout: VaultLayout::Recovery(VaultRecoveryConfig {
            custody: VaultSigners::new(vec![VaultMember::bls(BlsPair::new(1).pk)], 1),
            recovery_timelock: 60,
            version: VaultVersion::ForceSingletonRecreation,
            recovery_custody_hash: Bytes32::new([4; 32]),
            amount: 1,
        }),
        source: VaultConfigSource::Memo,
    })
    .await?;
    tx.commit().await?;

    assert_eq!(db.asset_kind(launcher_id).await?, Some(AssetKind::Vault));
    assert_eq!(db.vault_launcher_ids().await?, [launcher_id]);
    assert_eq!(db.unsynced_vault_launcher_ids().await?, [launcher_id]);

    let status = db.vault_status(launcher_id).await?.unwrap();
    assert_eq!(status.state, VaultState::Unsynced);
    assert_eq!(status.coin, None);

    // The eve coin's custody isn't known until it's spent, or its launcher reveals it.
    let eve = Coin::new(launcher_id, Bytes32::new([5; 32]), 1);
    insert_vault_coin(&db, launcher_id, eve, None, Some(10)).await?;

    assert!(db.unsynced_vault_launcher_ids().await?.is_empty());
    let status = db.vault_status(launcher_id).await?.unwrap();
    assert_eq!(status.state, VaultState::Active);
    assert_eq!(status.coin.unwrap().coin, eve);
    assert_eq!(status.config, None);

    let mut tx = db.tx().await?;
    tx.insert_vault_coin(eve.coin_id(), Some(custody_hash))
        .await?;
    tx.insert_vault_coin(eve.coin_id(), None).await?;
    tx.commit().await?;
    assert_eq!(
        db.vault_status(launcher_id)
            .await?
            .unwrap()
            .coin
            .unwrap()
            .custody_hash,
        Some(custody_hash)
    );

    // Starting recovery, and then recreating the vault while it's pending, keeps the time it
    // started.
    set_coin_state(&db, eve, Some(10), Some(11)).await?;
    let started = Coin::new(eve.coin_id(), Bytes32::new([6; 32]), 1);
    insert_vault_coin(&db, launcher_id, started, Some(recovery_hash), Some(11)).await?;
    set_coin_state(&db, started, Some(11), Some(12)).await?;
    let recreated = Coin::new(started.coin_id(), Bytes32::new([6; 32]), 3);
    insert_vault_coin(&db, launcher_id, recreated, Some(recovery_hash), Some(12)).await?;

    let status = db.vault_status(launcher_id).await?.unwrap();
    assert_eq!(
        status.state,
        VaultState::RecoveryPending {
            started_at: Some(1100)
        }
    );
    assert_eq!(status.coin.unwrap().coin, recreated);
    assert_eq!(status.config.unwrap().custody_hash, recovery_hash);
    assert_eq!(
        db.vault_coins(launcher_id)
            .await?
            .into_iter()
            .map(|row| row.coin)
            .collect::<Vec<_>>(),
        [eve, started, recreated]
    );

    // A reorg which undoes the recreation makes its parent current again.
    set_coin_state(&db, started, Some(11), None).await?;
    set_coin_state(&db, recreated, None, None).await?;
    assert_eq!(
        db.vault_status(launcher_id)
            .await?
            .unwrap()
            .coin
            .unwrap()
            .coin,
        started
    );

    // A spent coin is current until its children are synced. Without a singleton child, the vault
    // was melted.
    set_coin_state(&db, started, Some(11), Some(13)).await?;
    let status = db.vault_status(launcher_id).await?.unwrap();
    assert!(matches!(status.state, VaultState::RecoveryPending { .. }));

    let mut tx = db.tx().await?;
    tx.set_children_synced(started.coin_id()).await?;
    tx.delete_coin(recreated.coin_id()).await?;
    tx.commit().await?;

    let status = db.vault_status(launcher_id).await?.unwrap();
    assert_eq!(status.state, VaultState::Melted);
    assert_eq!(status.coin.unwrap().coin, started);

    Ok(())
}

#[tokio::test]
async fn stores_p2_singletons() -> anyhow::Result<()> {
    let db = migrated_db().await?;
    let launcher_id = Bytes32::new([1; 32]);

    let p2_singletons = [
        P2Singleton::new(launcher_id, 1),
        P2Singleton::new(launcher_id, 0),
    ];
    let p2_puzzle_hashes: Vec<Bytes32> = p2_singletons
        .iter()
        .map(|p2| p2.tree_hash().into())
        .collect();

    let mut tx = db.tx().await?;
    for p2_singleton in p2_singletons {
        tx.insert_p2_singleton_p2_puzzle(p2_singleton).await?;
    }
    tx.commit().await?;

    // They're custody, and synced from the start of the chain.
    for (p2_singleton, p2_puzzle_hash) in p2_singletons.into_iter().zip(&p2_puzzle_hashes) {
        assert!(db.is_custody_p2_puzzle_hash(*p2_puzzle_hash).await?);
        assert!(matches!(
            db.p2_puzzle(*p2_puzzle_hash).await?,
            P2Puzzle::P2Singleton(p2) if p2 == p2_singleton
        ));
    }

    let mut unsynced = db.unsynced_custody_p2_puzzle_hashes().await?;
    unsynced.sort();
    let mut expected = p2_puzzle_hashes.clone();
    expected.sort();
    assert_eq!(unsynced, expected);

    // Ordered by nonce.
    assert_eq!(
        db.p2_singleton_puzzle_hashes(launcher_id).await?,
        [p2_puzzle_hashes[1], p2_puzzle_hashes[0]]
    );

    Ok(())
}

#[tokio::test]
async fn stores_p2_conditions_or_singletons() -> anyhow::Result<()> {
    let db = migrated_db().await?;
    let launcher_id = Bytes32::new([1; 32]);

    let conditions = Program::from(vec![0x80]);
    let p2 = P2ConditionsOrSingleton::from_quoted_conditions_hash(
        launcher_id,
        0,
        Bytes32::new([2; 32]),
        P2ConditionsOrSingletonVersion::FixedPuzzleMember,
    );
    let p2_puzzle_hash: Bytes32 = p2.tree_hash().into();

    let mut tx = db.tx().await?;
    tx.insert_p2_conditions_or_singleton_p2_puzzle(p2, None)
        .await?;
    tx.commit().await?;

    assert!(!db.is_custody_p2_puzzle_hash(p2_puzzle_hash).await?);
    assert!(matches!(
        db.p2_puzzle(p2_puzzle_hash).await?,
        P2Puzzle::P2ConditionsOrSingleton(stored) if stored.p2 == p2 && stored.version.is_none()
    ));
    assert_eq!(
        db.p2_conditions_or_singleton_conditions(p2_puzzle_hash)
            .await?,
        None
    );

    // Revealing the conditions later fills them in.
    let mut tx = db.tx().await?;
    tx.insert_p2_conditions_or_singleton_p2_puzzle(
        p2,
        Some((
            P2ConditionsOrSingletonVersion::FixedPuzzleMember,
            conditions.clone(),
        )),
    )
    .await?;
    tx.insert_p2_conditions_or_singleton_p2_puzzle(p2, None)
        .await?;
    tx.commit().await?;

    assert!(matches!(
        db.p2_puzzle(p2_puzzle_hash).await?,
        P2Puzzle::P2ConditionsOrSingleton(stored)
            if stored.version == Some(P2ConditionsOrSingletonVersion::FixedPuzzleMember)
    ));
    assert_eq!(
        db.p2_conditions_or_singleton_conditions(p2_puzzle_hash)
            .await?,
        Some(conditions)
    );

    Ok(())
}

#[tokio::test]
async fn vault_coins_and_subscriptions() -> anyhow::Result<()> {
    let db = migrated_db().await?;
    let launcher_id = Bytes32::new([1; 32]);

    let p2_singleton = P2Singleton::new(launcher_id, 0);
    let p2_singleton_hash: Bytes32 = p2_singleton.tree_hash().into();
    let p2_conditions = P2ConditionsOrSingleton::new(launcher_id, 0, Bytes32::new([2; 32]));
    let p2_conditions_hash: Bytes32 = p2_conditions.tree_hash().into();

    let vault_coin = Coin::new(Bytes32::new([3; 32]), Bytes32::new([4; 32]), 1);
    let xch_coin = Coin::new(Bytes32::new([5; 32]), p2_singleton_hash, 100);
    let locked_coin = Coin::new(Bytes32::new([6; 32]), p2_conditions_hash, 200);

    let mut tx = db.tx().await?;
    tx.insert_height(10).await?;
    tx.insert_vault(launcher_id).await?;
    tx.insert_p2_singleton_p2_puzzle(p2_singleton).await?;
    tx.insert_p2_conditions_or_singleton_p2_puzzle(p2_conditions, None)
        .await?;

    for coin in [vault_coin, xch_coin, locked_coin] {
        tx.insert_coin(CoinState::new(coin, None, Some(10))).await?;
    }

    // The vault singleton has no p2 puzzle of its own.
    tx.update_coin(vault_coin.coin_id(), launcher_id, Bytes32::default())
        .await?;
    tx.update_coin(xch_coin.coin_id(), Bytes32::default(), p2_singleton_hash)
        .await?;
    tx.update_coin(
        locked_coin.coin_id(),
        Bytes32::default(),
        p2_conditions_hash,
    )
    .await?;
    tx.commit().await?;

    assert_eq!(
        db.coin_kind(vault_coin.coin_id()).await?,
        Some(CoinKind::Vault)
    );

    // XCH at the vault's address is found by puzzle hash, and the vault singleton and locked coins
    // by coin id.
    let mut subscriptions = db.subscription_coin_ids().await?;
    subscriptions.sort();
    let mut expected = vec![vault_coin.coin_id(), locked_coin.coin_id()];
    expected.sort();
    assert_eq!(subscriptions, expected);

    // Locked coins are only spendable by the vault once the offer is cancelled off-chain, so
    // they're selectable like any other coin.
    assert_eq!(db.xch_balance().await?, 300);
    let mut selectable = db.selectable_xch_coins().await?;
    selectable.sort_by_key(Coin::coin_id);
    let mut expected = vec![xch_coin, locked_coin];
    expected.sort_by_key(Coin::coin_id);
    assert_eq!(selectable, expected);

    Ok(())
}
