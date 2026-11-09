use chia_wallet_sdk::{
    chia::puzzle_types::nft::NftMetadata,
    prelude::*,
    puzzles::NFT_METADATA_UPDATER_DEFAULT_HASH,
    types::puzzles::{ANY_METADATA_UPDATER_HASH, HandleNftMetadata},
};
use sqlx::{SqlitePool, migrate, raw_sql};

use crate::{Database, NftUpdaterKind};

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
