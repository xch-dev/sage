/* 
  stand alone tables from the current schema 
  these are largely unchanged 
*/
CREATE TABLE future_did_names (
    launcher_id BLOB NOT NULL PRIMARY KEY,
    name TEXT NOT NULL
);

CREATE TABLE rust_migrations (
  version INTEGER PRIMARY KEY
);

/* 
  new tables that redefine the current schema and introduce the following conventions
  - all BOOLEAN columns are named is_<name>
  - all foreign keys are specified with FOREIGN KEY (and indexed)
  - all UNIX timestamps are INTEGER and named <name>_timestamp
  - except for blocks and rust_migrations, all tables have a surrogate primary key
  - all natural keys are specified as UNIQUE (which also creates an auto-index)
*/

/*
 * A single table that represents all kinds of supported assets on the Chia blockchain:
 * XCH = 0
 * CAT = 1
 * NFT = 2
 * DID = 3
 * Option = 4
 *
 * The hash represents the asset's unique on-chain identifier (asset id or launcher id).
 * Everything else is for display purposes only
 *
 * Note: For CATs, there isn't a defined created or spent height for the whole asset class,
 * but for singletons it's possible to no longer own the asset, but still reference it in
 * transaction history or offers.
 */
CREATE TABLE assets (
  id INTEGER PRIMARY KEY,
  hash BLOB NOT NULL UNIQUE,
  kind INTEGER NOT NULL,
  name TEXT,
  icon_url TEXT,
  description TEXT,
  is_visible BOOLEAN NOT NULL,
  is_pending BOOLEAN NOT NULL,
  created_height INTEGER
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
  height INTEGER PRIMARY KEY,
  header_hash BLOB,
  timestamp INTEGER
);

/*
 * A table of all p2 puzzle hashes that belong to the wallet, from kinds such as:
 * P2_DELEGATED_PUZZLE_OR_HIDDEN_PUZZLE = 0
 * P2_DELEGATED_PUZZLE = 1
 * CLAWBACK = 2
 * OPTION_UNDERLYING = 3
 *
 * However, outer puzzles such as the CAT or revocation layer are stored elsewhere.
 */
CREATE TABLE p2_puzzles (
  id INTEGER PRIMARY KEY,
  hash BLOB NOT NULL UNIQUE,
  kind INTEGER NOT NULL
);

/*
 * A table of all synthetic keys that pertain to p2 puzzles.
 * This is specifically for P2_DELEGATED_PUZZLE_OR_HIDDEN_PUZZLE and P2_DELEGATED_PUZZLE.
 * The id is the derivation index of the key.
 */
CREATE TABLE public_keys (
  id INTEGER PRIMARY KEY,
  p2_puzzle_id INTEGER NOT NULL,
  is_hardened BOOLEAN NOT NULL,
  is_synthetic BOOLEAN NOT NULL,
  key BLOB NOT NULL,
  FOREIGN KEY (p2_puzzle_id) REFERENCES p2_puzzles(id) ON DELETE CASCADE
);

CREATE TABLE clawbacks (
  id INTEGER PRIMARY KEY,
  p2_puzzle_id INTEGER NOT NULL,
  sender_puzzle_hash BLOB NOT NULL,
  receiver_puzzle_hash BLOB NOT NULL,
  seconds INTEGER NOT NULL,
  FOREIGN KEY (p2_puzzle_id) REFERENCES p2_puzzles(id) ON DELETE CASCADE
);

CREATE TABLE p2_options (
  id INTEGER PRIMARY KEY,
  p2_puzzle_id INTEGER NOT NULL,
  hash BLOB NOT NULL,
  FOREIGN KEY (p2_puzzle_id) REFERENCES p2_puzzles(id) ON DELETE CASCADE
);

/*
 * This is all coins which have been synced from the blockchain, created in a transaction,
 * or spent in a transaction.
 *
 * When a coin is discovered, and hasn't been synced yet, it's added to this table without
 * an asset_id. This will put it into a queue for further processing, in which case we
 * will lookup the coin on-chain and discover the asset.
 *
 * The hint is for identifying 
 */
CREATE TABLE coins (
  id INTEGER PRIMARY KEY,
  asset_id INTEGER,
  hash BLOB NOT NULL UNIQUE,
  parent_coin_id BLOB NOT NULL,
  puzzle_hash BLOB NOT NULL,
  amount BLOB NOT NULL,
  is_synced BOOLEAN NOT NULL,
  p2_puzzle_id INTEGER,
  memos BLOB,
  created_height INTEGER,
  spent_height INTEGER,
  FOREIGN KEY (asset_id) REFERENCES assets(id) ON DELETE CASCADE,
  FOREIGN KEY (p2_puzzle_id) REFERENCES p2_puzzles(id) ON DELETE SET NULL,
  FOREIGN KEY (created_height) REFERENCES blocks(height) ON DELETE CASCADE,
  FOREIGN KEY (spent_height) REFERENCES blocks(height) ON DELETE SET NULL
);

CREATE TABLE lineage_proofs (
  id INTEGER PRIMARY KEY,
  coin_id INTEGER NOT NULL UNIQUE,
  parent_parent_coin_id BLOB NOT NULL,
  parent_inner_puzzle_hash BLOB NOT NULL,
  parent_amount BLOB NOT NULL,
  FOREIGN KEY (coin_id) REFERENCES coins(id) ON DELETE CASCADE
);

/*
  Offer statuses
    Active = 0,
    Completed = 1,
    Cancelled = 2,
    Expired = 3,
*/
CREATE TABLE offers (
  id INTEGER PRIMARY KEY,
  hash BLOB NOT NULL UNIQUE,
  encoded_offer TEXT NOT NULL,
  fee BLOB NOT NULL,
  status INTEGER NOT NULL,
  expiration_height INTEGER,
  expiration_timestamp INTEGER,
  inserted_timestamp INTEGER NOT NULL
);

CREATE TABLE offer_assets (
  id INTEGER PRIMARY KEY,
  offer_id INTEGER NOT NULL,
  asset_id INTEGER NOT NULL,
  amount BLOB NOT NULL,
  royalty BLOB,
  is_requested BOOLEAN NOT NULL,
  FOREIGN KEY (offer_id) REFERENCES offers(id) ON DELETE CASCADE,
  FOREIGN KEY (asset_id) REFERENCES assets(id) ON DELETE CASCADE,
  UNIQUE(offer_id, asset_id)
);

CREATE TABLE transactions (
  id INTEGER PRIMARY KEY,
  hash BLOB NOT NULL UNIQUE,
  aggregated_signature BLOB,
  fee BLOB,
  height INTEGER,
  submitted_at_timestamp INTEGER,
  FOREIGN KEY (height) REFERENCES blocks(height) ON DELETE CASCADE
);

CREATE TABLE transaction_coins (
  id INTEGER PRIMARY KEY,
  transaction_id INTEGER NOT NULL,
  coin_id INTEGER NOT NULL UNIQUE,
  'index' INTEGER NOT NULL,
  puzzle_hash BLOB NOT NULL,
  puzzle_reveal BLOB NOT NULL,
  solution BLOB NOT NULL,
  is_spend BOOLEAN NOT NULL,
  FOREIGN KEY (transaction_id) REFERENCES transactions(id) ON DELETE CASCADE,
  UNIQUE(transaction_id, coin_id)
);

CREATE TABLE collections (
  id INTEGER PRIMARY KEY,
  name TEXT,
  hash BLOB NOT NULL UNIQUE,
  description TEXT,
  metadata_id TEXT NOT NULL,
  is_visible BOOLEAN NOT NULL,
  minter_did BLOB NOT NULL,
  icon_url TEXT,
  banner_url TEXT
);

CREATE TABLE nfts (
  id INTEGER PRIMARY KEY,
  asset_id INTEGER NOT NULL UNIQUE,
  collection_id INTEGER,
  minter_did BLOB,
  owner_did BLOB,
  current_owner BLOB,
  is_owned BOOLEAN NOT NULL,
  is_sensitive_content BOOLEAN NOT NULL DEFAULT FALSE,
  metadata BLOB,
  metadata_updater_puzzle_hash BLOB,
  royalty_ten_thousandths INTEGER,
  royalty_puzzle_hash BLOB,
  metadata_hash BLOB,
  data_hash BLOB NOT NULL,
  license_hash BLOB NOT NULL,
  edition_number INTEGER,
  edition_total INTEGER,
  FOREIGN KEY (collection_id) REFERENCES collections(id) ON DELETE SET NULL,
  FOREIGN KEY (asset_id) REFERENCES assets(id) ON DELETE CASCADE
);

/* 
  This table collapses nft_data, nft_uris, and nft_thumbnails into a single table
  with kind to differentiate between the three types of data.
  Also, data_index, is a pointer to an external data source. It could
  be a file path, a cache index, or a url etc so need to flesh that out more.

  kind values:
    - 0 = data
    - 1 = uri
    - 2 = thumbnail
    - 3 = icon
*/
CREATE TABLE nft_data (
  id INTEGER PRIMARY KEY,
  nft_id INTEGER NOT NULL,
  kind INTEGER NOT NULL,
  mime_type TEXT,
  is_hash_matched BOOLEAN NOT NULL,
  data_index TEXT NOT NULL, 
  FOREIGN KEY (nft_id) REFERENCES nfts(id) ON DELETE CASCADE
);

CREATE TABLE cats (
  id INTEGER PRIMARY KEY,
  asset_id INTEGER NOT NULL UNIQUE,
  ticker TEXT,
  FOREIGN KEY (asset_id) REFERENCES assets(id) ON DELETE CASCADE
);

CREATE TABLE dids (
  id INTEGER PRIMARY KEY,
  asset_id INTEGER NOT NULL UNIQUE,
  is_owned BOOLEAN NOT NULL,
  metadata BLOB NOT NULL,
  recovery_list_hash BLOB,
  num_verifications_required BLOB NOT NULL,
  FOREIGN KEY (asset_id) REFERENCES assets(id) ON DELETE CASCADE
);

CREATE TABLE offer_coins (
  id INTEGER PRIMARY KEY,
  offer_id INTEGER NOT NULL,
  coin_id INTEGER NOT NULL,
  FOREIGN KEY (offer_id) REFERENCES offers(id) ON DELETE CASCADE
  FOREIGN KEY (coin_id) REFERENCES coins(id) ON DELETE CASCADE
  UNIQUE(offer_id, coin_id)
);
