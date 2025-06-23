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

/*
 * A single table that represents all kinds of supported assets on the Chia blockchain:
 * Token = 0
 * NFT = 1
 * DID = 2
 * Option = 3
 *
 * The hash represents the asset's unique on-chain identifier (asset id or launcher id).
 * Everything else is for display purposes only
 *
 * Note: For CATs, there isn't a defined created or spent height for the whole asset class,
 * but for singletons it's possible to no longer own the asset, but still reference it in
 * transaction history or offers.
 */
CREATE TABLE assets (
  id INTEGER NOT NULL PRIMARY KEY,
  hash BLOB NOT NULL UNIQUE,
  kind INTEGER NOT NULL,
  name TEXT,
  icon_url TEXT,
  description TEXT,
  is_visible BOOLEAN NOT NULL,
  is_pending BOOLEAN NOT NULL,
  created_height INTEGER
);

CREATE TABLE tokens (
  id INTEGER NOT NULL PRIMARY KEY,
  asset_id INTEGER NOT NULL UNIQUE,
  ticker TEXT,
  precision INTEGER NOT NULL DEFAULT 3,
  FOREIGN KEY (asset_id) REFERENCES assets(id) ON DELETE CASCADE
);

CREATE TABLE nfts (
  id INTEGER NOT NULL PRIMARY KEY,
  asset_id INTEGER NOT NULL UNIQUE,
  collection_id INTEGER,
  minter_hash BLOB,
  owner_hash BLOB,
  is_sensitive_content BOOLEAN NOT NULL DEFAULT FALSE,
  metadata BLOB NOT NULL,
  metadata_updater_puzzle_hash BLOB NOT NULL,
  royalty_puzzle_hash BLOB NOT NULL,
  royalty_basis_points INTEGER NOT NULL,
  data_hash BLOB,
  metadata_hash BLOB,
  license_hash BLOB,
  edition_number INTEGER,
  edition_total INTEGER,
  FOREIGN KEY (collection_id) REFERENCES collections(id) ON DELETE SET NULL,
  FOREIGN KEY (asset_id) REFERENCES assets(id) ON DELETE CASCADE
);

CREATE TABLE dids (
  id INTEGER NOT NULL PRIMARY KEY,
  asset_id INTEGER NOT NULL UNIQUE,
  metadata BLOB NOT NULL,
  recovery_list_hash BLOB,
  num_verifications_required BLOB NOT NULL,
  FOREIGN KEY (asset_id) REFERENCES assets(id) ON DELETE CASCADE
);

CREATE TABLE options (
  id INTEGER NOT NULL PRIMARY KEY,
  asset_id INTEGER NOT NULL UNIQUE,
  creator_puzzle_hash BLOB NOT NULL,
  expiration_seconds INTEGER NOT NULL,
  underlying_asset_id INTEGER NOT NULL,
  underlying_amount BLOB NOT NULL,
  underlying_coin_hash BLOB NOT NULL,
  underlying_delegated_puzzle_hash BLOB NOT NULL,
  strike_asset_id INTEGER NOT NULL,
  strike_hidden_puzzle_hash BLOB,
  strike_settlement_puzzle_hash BLOB,
  strike_amount BLOB NOT NULL,
  FOREIGN KEY (asset_id) REFERENCES assets(id) ON DELETE CASCADE,
  FOREIGN KEY (underlying_asset_id) REFERENCES assets(id) ON DELETE CASCADE,
  FOREIGN KEY (strike_asset_id) REFERENCES assets(id) ON DELETE CASCADE
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
  timestamp INTEGER
);

/*
 * A table of all p2 puzzle hashes that belong to the wallet, from kinds such as:
 * Derivation = 0
 * Clawback = 1
 * OptionUnderlying = 2
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
  p2_puzzle_id INTEGER NOT NULL,
  is_hardened BOOLEAN NOT NULL,
  derivation_index INTEGER NOT NULL,
  key BLOB NOT NULL,
  FOREIGN KEY (p2_puzzle_id) REFERENCES p2_puzzles(id) ON DELETE CASCADE
);

CREATE TABLE clawbacks (
  id INTEGER NOT NULL PRIMARY KEY,
  p2_puzzle_id INTEGER NOT NULL,
  sender_puzzle_hash BLOB NOT NULL,
  receiver_puzzle_hash BLOB NOT NULL,
  expiration_seconds INTEGER NOT NULL,
  FOREIGN KEY (p2_puzzle_id) REFERENCES p2_puzzles(id) ON DELETE CASCADE
);

CREATE TABLE p2_options (
  id INTEGER NOT NULL PRIMARY KEY,
  p2_puzzle_id INTEGER NOT NULL,
  option_asset_id INTEGER NULL,
  FOREIGN KEY (p2_puzzle_id) REFERENCES p2_puzzles(id) ON DELETE CASCADE,
  FOREIGN KEY (option_asset_id) REFERENCES assets(id) ON DELETE CASCADE
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
  id INTEGER NOT NULL PRIMARY KEY,
  asset_id INTEGER,
  hash BLOB NOT NULL UNIQUE,
  parent_coin_hash BLOB NOT NULL,
  puzzle_hash BLOB NOT NULL,
  amount BLOB NOT NULL,
  hidden_puzzle_hash BLOB,
  p2_puzzle_id INTEGER,
  created_height INTEGER,
  spent_height INTEGER,
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
  FOREIGN KEY (offer_id) REFERENCES offers(id) ON DELETE CASCADE
  FOREIGN KEY (coin_id) REFERENCES coins(id) ON DELETE CASCADE
  UNIQUE(offer_id, coin_id)
);

CREATE TABLE transactions (
  id INTEGER NOT NULL PRIMARY KEY,
  hash BLOB NOT NULL UNIQUE,
  aggregated_signature BLOB,
  fee BLOB NOT NULL,
  submitted_timestamp INTEGER NOT NULL DEFAULT (unixepoch())
);

CREATE TABLE transaction_coins (
  id INTEGER NOT NULL PRIMARY KEY,
  transaction_id INTEGER NOT NULL,
  asset_id INTEGER NOT NULL,
  coin_id INTEGER,
  coin_hash BLOB NOT NULL,
  parent_coin_hash BLOB NOT NULL,
  puzzle_hash BLOB NOT NULL,
  amount BLOB NOT NULL,
  is_output BOOLEAN NOT NULL,
  seq INTEGER NOT NULL,
  FOREIGN KEY (transaction_id) REFERENCES transactions(id) ON DELETE CASCADE,
  FOREIGN KEY (asset_id) REFERENCES assets(id) ON DELETE SET NULL,
  FOREIGN KEY (coin_id) REFERENCES coins(id) ON DELETE SET NULL,
  UNIQUE(transaction_id, coin_hash)
);

CREATE TABLE transaction_spends (
  id INTEGER NOT NULL PRIMARY KEY,
  transaction_coin_id INTEGER NOT NULL UNIQUE,
  puzzle_reveal BLOB NOT NULL,
  solution BLOB NOT NULL,
  FOREIGN KEY (transaction_coin_id) REFERENCES transaction_coins(id) ON DELETE CASCADE
);

CREATE TABLE collections (
  id INTEGER NOT NULL PRIMARY KEY,
  hash BLOB NOT NULL UNIQUE,
  uuid TEXT NOT NULL,
  minter_hash BLOB NOT NULL,
  name TEXT,
  icon_url TEXT,
  banner_url TEXT,
  description TEXT,
  is_visible BOOLEAN NOT NULL,
  created_height INTEGER
);

CREATE TABLE files (
  id INTEGER NOT NULL PRIMARY KEY,
  hash BLOB NOT NULL UNIQUE,
  mime_type TEXT,
  is_hash_match BOOLEAN NOT NULL,
  is_downloaded BOOLEAN NOT NULL
);

CREATE TABLE file_uris (
  id INTEGER NOT NULL PRIMARY KEY,
  file_id INTEGER NOT NULL,
  uri TEXT NOT NULL,
  FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE,
  UNIQUE(file_id, uri)
);
