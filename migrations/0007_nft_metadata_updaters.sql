/*
 * The kind of metadata updater puzzle an NFT uses:
 * Default = 0 (the CHIP-7 updater, which can only append URIs)
 * Any = 1 (the XCHandles updater, which lets the owner replace all metadata)
 * Unknown = 2
 *
 * New NFTs always have the kind computed from metadata_updater_puzzle_hash on insert.
 */
ALTER TABLE nfts ADD COLUMN metadata_updater_kind INTEGER NOT NULL DEFAULT 2;

/*
 * The owner-editable display name (`dn`) from XCHandles metadata. This is a display value
 * and lookup hint only, and doesn't prove ownership of a handle.
 *
 * It's backfilled for existing NFTs by the version 2 Rust migration, since parsing it requires CLVM.
 */
ALTER TABLE nfts ADD COLUMN display_name TEXT;

UPDATE nfts SET metadata_updater_kind = CASE metadata_updater_puzzle_hash
  WHEN x'fe8a4b4e27a2e29a4d3fc7ce9d527adbcaccbab6ada3903ccf3ba9a769d2d78b' THEN 0
  WHEN x'9f28d55242a3bd2b3661c38ba8647392c26bb86594050ea6d33aad1725ca3eea' THEN 1
  ELSE 2
END;

CREATE INDEX idx_nfts_metadata_updater_kind ON nfts(metadata_updater_kind);

DROP VIEW owned_nfts;
DROP VIEW wallet_nfts;

CREATE VIEW owned_nfts AS
  SELECT
      owned_coins.*, nfts.minter_hash, owner_hash, metadata, metadata_updater_puzzle_hash,
      metadata_updater_kind, display_name,
      royalty_puzzle_hash, royalty_basis_points, data_hash, metadata_hash, license_hash,
      edition_number, edition_total, nfts.collection_id
  FROM owned_coins
  INNER JOIN nfts ON nfts.asset_id = owned_coins.asset_id;

CREATE VIEW wallet_nfts AS
  SELECT
      wallet_coins.*, nfts.minter_hash, owner_hash, metadata, metadata_updater_puzzle_hash,
      metadata_updater_kind, display_name,
      royalty_puzzle_hash, royalty_basis_points, data_hash, metadata_hash, license_hash,
      edition_number, edition_total, nfts.collection_id
  FROM wallet_coins
  INNER JOIN nfts ON nfts.asset_id = wallet_coins.asset_id;
