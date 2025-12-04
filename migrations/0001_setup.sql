PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;

/* 
 * The following conventions are used in tables:
 *
 * 1. All BOOLEAN columns are named is_<name>
 * 2. All foreign keys are specified with FOREIGN KEY (and indexed)
 * 3. All UNIX timestamps are INTEGER and named <name>_timestamp
 * 4. All tables have a surrogate or INTEGER NOT NULL PRIMARY KEY
 * 5. All natural keys are specified as UNIQUE (which also creates an auto-index)
*/

/*
 * A table with one row that represents changes made so far to the schema that involve
 * more complex or parameterizable code than SQLite can handle.
 */
CREATE TABLE rust_migrations (
  version INTEGER NOT NULL PRIMARY KEY
);

INSERT INTO rust_migrations (version) VALUES (0);

CREATE TABLE collections (
  id INTEGER NOT NULL PRIMARY KEY,
  hash BLOB NOT NULL UNIQUE,
  uuid TEXT NOT NULL,
  minter_hash BLOB NOT NULL,
  name TEXT,
  icon_url TEXT,
  banner_url TEXT,
  description TEXT,
  is_visible BOOLEAN NOT NULL
);

INSERT INTO collections (id, name, hash, uuid, minter_hash, is_visible)
VALUES (0, 'No Collection', x'0000000000000000000000000000000000000000000000000000000000000000', '00000000-0000-0000-0000-000000000000', x'0000000000000000000000000000000000000000000000000000000000000000', TRUE);

CREATE TRIGGER trigger_prevent_delete_default_collection
    BEFORE DELETE ON collections
    FOR EACH ROW
    WHEN OLD.id = 0
BEGIN
    SELECT RAISE(ABORT, 'Cannot delete default collection');
END;

/*
 * A single table that represents all kinds of supported assets on the Chia blockchain:
 * Token = 0
 * NFT = 1
 * DID = 2
 *
 * The hash represents the asset's unique on-chain identifier (asset id or launcher id).
 * Everything else is for display purposes only
 */
CREATE TABLE assets (
  id INTEGER NOT NULL PRIMARY KEY,
  hash BLOB NOT NULL UNIQUE,
  kind INTEGER NOT NULL,
  name TEXT,
  ticker TEXT,
  precision INTEGER NOT NULL,
  icon_url TEXT,
  description TEXT,
  is_sensitive_content BOOLEAN NOT NULL DEFAULT FALSE,
  is_visible BOOLEAN NOT NULL,
  hidden_puzzle_hash BLOB
);

INSERT INTO assets (id, kind, hash, name, ticker, precision, description, icon_url, is_visible, hidden_puzzle_hash) 
VALUES (0, 0, x'0000000000000000000000000000000000000000000000000000000000000000', 'Chia', 'XCH', 12, 'The primary asset on the Chia blockchain.', 'https://icons.dexie.space/xch.webp', TRUE, NULL);

CREATE TRIGGER trigger_prevent_delete_xch_asset
    BEFORE DELETE ON assets
    FOR EACH ROW
    WHEN OLD.id = 0
BEGIN
    SELECT RAISE(ABORT, 'Cannot delete XCH asset');
END;

CREATE TABLE nfts (
  id INTEGER NOT NULL PRIMARY KEY,
  asset_id INTEGER NOT NULL UNIQUE,
  collection_id INTEGER NOT NULL DEFAULT 0,
  minter_hash BLOB,
  owner_hash BLOB,
  metadata BLOB NOT NULL,
  metadata_updater_puzzle_hash BLOB NOT NULL,
  royalty_puzzle_hash BLOB NOT NULL,
  royalty_basis_points INTEGER NOT NULL,
  data_hash BLOB,
  metadata_hash BLOB,
  license_hash BLOB,
  edition_number INTEGER,
  edition_total INTEGER,
  FOREIGN KEY (collection_id) REFERENCES collections(id) ON DELETE SET DEFAULT,
  FOREIGN KEY (asset_id) REFERENCES assets(id) ON DELETE CASCADE
);

CREATE TABLE dids (
  id INTEGER NOT NULL PRIMARY KEY,
  asset_id INTEGER NOT NULL UNIQUE,
  metadata BLOB NOT NULL,
  recovery_list_hash BLOB,
  num_verifications_required INTEGER NOT NULL,
  FOREIGN KEY (asset_id) REFERENCES assets(id) ON DELETE CASCADE
);

/*
 * This isn't a comprehensive history of the blockchain, but keeps track of blocks that
 * have been synced. It's primarily used for identifying the timestamp in which coins
 * were created or spent.
 *
 * However, its secondary use is for performing a rollback if the header hash of a
 * previous block is changed (a fork).
 */
CREATE TABLE blocks (
  height INTEGER NOT NULL PRIMARY KEY,
  header_hash BLOB,
  timestamp INTEGER,
  is_peak BOOLEAN NOT NULL
);

/*
 * A table of all p2 puzzle hashes that belong to the wallet, from kinds such as:
 * Derivation = 0
 * Clawback = 1
 *
 * However, outer puzzles such as the CAT or revocation layer are stored elsewhere.
 */
CREATE TABLE p2_puzzles (
  id INTEGER NOT NULL PRIMARY KEY,
  hash BLOB NOT NULL UNIQUE,
  kind INTEGER NOT NULL
);

CREATE TABLE public_keys (
  id INTEGER NOT NULL PRIMARY KEY,
  p2_puzzle_id INTEGER NOT NULL UNIQUE,
  is_hardened BOOLEAN NOT NULL,
  derivation_index INTEGER NOT NULL,
  key BLOB NOT NULL,
  FOREIGN KEY (p2_puzzle_id) REFERENCES p2_puzzles(id) ON DELETE CASCADE
);

CREATE TABLE clawbacks (
  id INTEGER NOT NULL PRIMARY KEY,
  p2_puzzle_id INTEGER NOT NULL UNIQUE,
  sender_puzzle_hash BLOB NOT NULL,
  receiver_puzzle_hash BLOB NOT NULL,
  expiration_seconds INTEGER NOT NULL,
  FOREIGN KEY (p2_puzzle_id) REFERENCES p2_puzzles(id) ON DELETE CASCADE
);

/*
 * This is all coins which have been synced from the blockchain, created in a transaction,
 * or spent in a transaction.
 *
 * When a coin is discovered, and hasn't been synced yet, it's added to this table without
 * an asset_id. This will put it into a queue for further processing, in which case we
 * will lookup the coin on-chain and discover the asset.
 */
CREATE TABLE coins (
  id INTEGER NOT NULL PRIMARY KEY,
  asset_id INTEGER,
  hash BLOB NOT NULL UNIQUE,
  parent_coin_hash BLOB NOT NULL,
  puzzle_hash BLOB NOT NULL,
  amount BLOB NOT NULL,
  p2_puzzle_id INTEGER,
  created_height INTEGER,
  spent_height INTEGER,
  is_children_synced BOOLEAN NOT NULL DEFAULT FALSE,
  FOREIGN KEY (asset_id) REFERENCES assets(id) ON DELETE CASCADE,
  FOREIGN KEY (p2_puzzle_id) REFERENCES p2_puzzles(id) ON DELETE SET NULL,
  FOREIGN KEY (created_height) REFERENCES blocks(height) ON DELETE CASCADE,
  FOREIGN KEY (spent_height) REFERENCES blocks(height) ON DELETE SET NULL
);

CREATE TABLE lineage_proofs (
  id INTEGER NOT NULL PRIMARY KEY,
  coin_id INTEGER NOT NULL UNIQUE,
  parent_parent_coin_hash BLOB NOT NULL,
  parent_inner_puzzle_hash BLOB NOT NULL,
  parent_amount BLOB NOT NULL,
  FOREIGN KEY (coin_id) REFERENCES coins(id) ON DELETE CASCADE
);

/*
 * Offer statuses
 *
 * Pending = 0
 * Active = 1
 * Completed = 2
 * Cancelled = 3
 * Expired = 4
 */
CREATE TABLE offers (
  id INTEGER NOT NULL PRIMARY KEY,
  hash BLOB NOT NULL UNIQUE,
  encoded_offer TEXT NOT NULL,
  fee BLOB NOT NULL,
  status INTEGER NOT NULL,
  expiration_height INTEGER,
  expiration_timestamp INTEGER,
  inserted_timestamp INTEGER NOT NULL DEFAULT (unixepoch())
);

CREATE TABLE offer_assets (
  id INTEGER NOT NULL PRIMARY KEY,
  offer_id INTEGER NOT NULL,
  asset_id INTEGER NOT NULL,
  is_requested BOOLEAN NOT NULL,
  amount BLOB NOT NULL,
  royalty BLOB NOT NULL,
  FOREIGN KEY (offer_id) REFERENCES offers(id) ON DELETE CASCADE,
  FOREIGN KEY (asset_id) REFERENCES assets(id) ON DELETE CASCADE,
  UNIQUE(offer_id, asset_id, is_requested)
);

CREATE TABLE offer_coins (
  id INTEGER NOT NULL PRIMARY KEY,
  offer_id INTEGER NOT NULL,
  coin_id INTEGER NOT NULL,
  FOREIGN KEY (offer_id) REFERENCES offers(id) ON DELETE CASCADE,
  FOREIGN KEY (coin_id) REFERENCES coins(id) ON DELETE CASCADE,
  UNIQUE(offer_id, coin_id)
);

CREATE TABLE mempool_items (
  id INTEGER NOT NULL PRIMARY KEY,
  hash BLOB NOT NULL UNIQUE,
  aggregated_signature BLOB NOT NULL,
  fee BLOB NOT NULL,
  submitted_timestamp INTEGER
);

CREATE TABLE mempool_coins (
  id INTEGER NOT NULL PRIMARY KEY,
  mempool_item_id INTEGER NOT NULL,
  coin_id INTEGER NOT NULL,
  is_input BOOLEAN NOT NULL,
  is_output BOOLEAN NOT NULL,
  FOREIGN KEY (mempool_item_id) REFERENCES mempool_items(id) ON DELETE CASCADE,
  FOREIGN KEY (coin_id) REFERENCES coins(id) ON DELETE CASCADE,
  UNIQUE(mempool_item_id, coin_id)
);

CREATE TABLE mempool_spends (
  id INTEGER NOT NULL PRIMARY KEY,
  mempool_item_id INTEGER NOT NULL,
  coin_hash BLOB NOT NULL,
  parent_coin_hash BLOB NOT NULL,
  puzzle_hash BLOB NOT NULL,
  amount BLOB NOT NULL,
  puzzle_reveal BLOB NOT NULL,
  solution BLOB NOT NULL,
  seq INTEGER NOT NULL,
  FOREIGN KEY (mempool_item_id) REFERENCES mempool_items(id) ON DELETE CASCADE,
  UNIQUE(mempool_item_id, coin_hash)
);

CREATE TABLE files (
  id INTEGER NOT NULL PRIMARY KEY,
  hash BLOB NOT NULL UNIQUE,
  data BLOB,
  mime_type TEXT,
  is_hash_match BOOLEAN
);

/*
 * Resized images
 *
 * Icon = 0
 * Thumbnail = 1
 */
CREATE TABLE resized_images (
  id INTEGER NOT NULL PRIMARY KEY,
  file_id INTEGER NOT NULL,
  kind INTEGER NOT NULL,
  data BLOB NOT NULL,
  FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE
);

CREATE TABLE file_uris (
  id INTEGER NOT NULL PRIMARY KEY,
  file_id INTEGER NOT NULL,
  uri TEXT NOT NULL,
  last_checked_timestamp INTEGER,
  failed_attempts INTEGER NOT NULL DEFAULT 0,
  FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE,
  UNIQUE(file_id, uri)
);

/* foreign key indices */

-- this index plays a dual role as an FK index and helps with the puzzle queue
CREATE INDEX idx_coins_asset_spent_children ON coins (asset_id, spent_height, is_children_synced);
CREATE INDEX idx_coins_p2_puzzle_id ON coins(p2_puzzle_id);
CREATE INDEX idx_coins_created_height ON coins(created_height);
CREATE INDEX idx_coins_spent_height ON coins(spent_height);
CREATE INDEX idx_file_uris_file_id ON file_uris(file_id);
CREATE INDEX idx_mempool_coins_coin_id ON mempool_coins(coin_id);
CREATE INDEX idx_mempool_coins_mempool_item_id ON mempool_coins(mempool_item_id);
CREATE INDEX idx_mempool_spends_mempool_item_id ON mempool_spends(mempool_item_id);
CREATE INDEX idx_nfts_data_hash ON nfts(data_hash);
CREATE INDEX idx_nfts_metadata_hash ON nfts(metadata_hash);
CREATE INDEX idx_nfts_license_hash ON nfts(license_hash);
CREATE INDEX idx_nfts_collection_id ON nfts(collection_id);
CREATE INDEX idx_offer_assets_offer_id ON offer_assets(offer_id);
CREATE INDEX idx_offer_assets_asset_id ON offer_assets(asset_id);
CREATE INDEX idx_offer_coins_offer_id ON offer_coins(offer_id);
CREATE INDEX idx_offer_coins_coin_id ON offer_coins(coin_id);
CREATE INDEX idx_resized_images_file_id ON resized_images(file_id);
CREATE INDEX idx_peaks ON blocks(is_peak DESC, height DESC);

/* search and ordering indices */
CREATE INDEX idx_assets_name ON assets(name ASC);
CREATE INDEX idx_assets_ticker ON assets(ticker);
CREATE INDEX idx_clawbacks_sender_puzzle_hash ON clawbacks(sender_puzzle_hash);
CREATE INDEX idx_clawbacks_receiver_puzzle_hash ON clawbacks(receiver_puzzle_hash);
CREATE INDEX idx_clawbacks_expiration_seconds ON clawbacks(expiration_seconds);
CREATE INDEX idx_nfts_minter_hash ON nfts(minter_hash);
CREATE INDEX idx_nfts_owner_hash ON nfts(owner_hash);
CREATE INDEX idx_nfts_edition_number ON nfts(edition_number ASC);
CREATE INDEX idx_public_keys_key ON public_keys(key);
CREATE INDEX idx_public_keys_derivation_index_hardened ON public_keys(derivation_index, is_hardened);

CREATE VIEW wallet_coins AS
SELECT
  coins.id AS coin_id,
  coins.hash AS coin_hash,
  coins.asset_id AS asset_id,
  coins.parent_coin_hash,
  coins.puzzle_hash,
  coins.amount,
  coins.p2_puzzle_id,
  coins.spent_height,
  coins.created_height,
  assets.hash AS asset_hash,
  assets.kind AS asset_kind,
  assets.name AS asset_name,
  assets.ticker AS asset_ticker,
  assets.precision AS asset_precision,
  assets.icon_url AS asset_icon_url,
  assets.description AS asset_description,
  assets.is_visible AS asset_is_visible,
  assets.is_sensitive_content AS asset_is_sensitive_content,
  assets.hidden_puzzle_hash AS asset_hidden_puzzle_hash,
  p2_puzzles.hash AS p2_puzzle_hash,
  p2_puzzles.kind AS p2_puzzle_kind,
  clawbacks.sender_puzzle_hash AS clawback_sender_puzzle_hash,
  clawbacks.receiver_puzzle_hash AS clawback_receiver_puzzle_hash,
  clawbacks.expiration_seconds AS clawback_expiration_seconds,
  sender_p2_puzzle.id AS clawback_sender_p2_puzzle_id,
  receiver_p2_puzzle.id AS clawback_receiver_p2_puzzle_id,
  created_blocks.timestamp AS created_timestamp,
  spent_blocks.timestamp AS spent_timestamp,
  (
    SELECT hash FROM mempool_items
    INNER JOIN mempool_coins ON mempool_coins.mempool_item_id = mempool_items.id
    WHERE mempool_coins.coin_id = coins.id
    AND mempool_coins.is_input = TRUE
    LIMIT 1
  ) AS mempool_item_hash,
  (
    SELECT hash FROM offers
    INNER JOIN offer_coins ON offer_coins.offer_id = offers.id
    WHERE offer_coins.coin_id = coins.id
    AND offers.status <= 1
    LIMIT 1
  ) AS offer_hash
FROM coins
  INNER JOIN assets ON assets.id = coins.asset_id
  INNER JOIN p2_puzzles ON p2_puzzles.id = coins.p2_puzzle_id
  LEFT JOIN clawbacks ON clawbacks.p2_puzzle_id = p2_puzzles.id
  LEFT JOIN p2_puzzles AS sender_p2_puzzle ON sender_p2_puzzle.hash = clawbacks.sender_puzzle_hash
  LEFT JOIN p2_puzzles AS receiver_p2_puzzle ON receiver_p2_puzzle.hash = clawbacks.receiver_puzzle_hash
  LEFT JOIN blocks AS created_blocks ON created_blocks.height = coins.created_height
  LEFT JOIN blocks AS spent_blocks ON spent_blocks.height = coins.spent_height;

CREATE VIEW selectable_coins AS
SELECT *
FROM wallet_coins
WHERE 1=1
  AND created_height IS NOT NULL
  AND spent_height IS NULL
  AND mempool_item_hash IS NULL
  AND offer_hash IS NULL
  AND NOT EXISTS (
    SELECT 1 FROM mempool_coins
    WHERE mempool_coins.coin_id = wallet_coins.coin_id
  )
  AND (
    clawback_expiration_seconds IS NULL
    OR (clawback_receiver_p2_puzzle_id IS NOT NULL AND unixepoch() >= clawback_expiration_seconds)
  );

CREATE VIEW owned_coins AS
SELECT *
FROM wallet_coins
WHERE 1=1
  AND spent_height IS NULL
  AND mempool_item_hash IS NULL
  AND (
    clawback_expiration_seconds IS NULL
    OR (clawback_receiver_p2_puzzle_id IS NOT NULL AND unixepoch() >= clawback_expiration_seconds)
  );

CREATE VIEW spent_coins AS
SELECT *
FROM wallet_coins
WHERE spent_height IS NOT NULL OR mempool_item_hash IS NOT NULL;

CREATE VIEW clawback_coins AS
SELECT *
FROM wallet_coins
WHERE 1=1
  AND spent_height IS NULL
  AND unixepoch() < clawback_expiration_seconds;

CREATE VIEW spendable_coins AS
SELECT *
FROM wallet_coins
WHERE 1=1
  AND spent_height IS NULL
  AND NOT EXISTS (
    SELECT 1 FROM mempool_coins
    WHERE mempool_coins.coin_id = wallet_coins.coin_id
  )
  AND (
    clawback_expiration_seconds IS NULL
    OR (clawback_receiver_p2_puzzle_id IS NOT NULL AND unixepoch() >= clawback_expiration_seconds)
    OR (clawback_sender_p2_puzzle_id IS NOT NULL AND unixepoch() < clawback_expiration_seconds)
  );

CREATE VIEW transaction_coins AS
SELECT
  blocks.height,
  blocks.timestamp,
  coins.hash AS coin_id,
  coins.puzzle_hash,
  coins.parent_coin_hash,
  coins.amount,
  coins.created_height = blocks.height AS is_created_in_block,
  coins.spent_height = blocks.height AS is_spent_in_block,
  p2_puzzles.hash AS p2_puzzle_hash,
  assets.hash AS asset_hash,
  assets.name AS asset_name,
  assets.ticker AS asset_ticker,
  assets.precision AS asset_precision,
  assets.icon_url AS asset_icon_url,
  assets.kind AS asset_kind,
  assets.description AS asset_description,
  assets.is_visible AS asset_is_visible,
  assets.is_sensitive_content AS asset_is_sensitive_content,
  assets.hidden_puzzle_hash AS asset_hidden_puzzle_hash
FROM blocks
LEFT JOIN coins ON coins.created_height = blocks.height OR coins.spent_height = blocks.height
INNER JOIN assets ON assets.id = coins.asset_id
LEFT JOIN p2_puzzles ON p2_puzzles.id = coins.p2_puzzle_id;

CREATE VIEW owned_nfts AS
  SELECT        
      owned_coins.*, nfts.minter_hash, owner_hash, metadata, metadata_updater_puzzle_hash,
      royalty_puzzle_hash, royalty_basis_points, data_hash, metadata_hash, license_hash,
      edition_number, edition_total, nfts.collection_id
  FROM owned_coins
  INNER JOIN nfts ON nfts.asset_id = owned_coins.asset_id;
