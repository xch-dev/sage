/* 
  stand alone tables from the current schema 
  these are largely unchanged 
*/
CREATE TABLE future_did_names (
    launcher_id BLOB NOT NULL PRIMARY KEY,
    name TEXT NOT NULL
);

CREATE TABLE derivations (
  id INTEGER PRIMARY KEY,
  p2_puzzle_hash BLOB NOT NULL UNIQUE,
  `index` INTEGER NOT NULL,
  hardened BOOLEAN NOT NULL,
  synthetic_key BLOB NOT NULL
);

CREATE TABLE rust_migrations (
  version INTEGER PRIMARY KEY
);

/* 
  new tables that redefine the current schema and introduce the following conventions
  - all BOOLEAN columns are named is_<name>
  - all foreign keys are specified with FOREIGN KEY (and indexed)
  - all UNIX timestamps are INTEGER and named <name>_timestamp
  - except for blocks, derivations and rust_migrations, all tables have a surrogate primary key
  - all natural keys are specified as UNIQUE (which also creates an auto-index)
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

CREATE TABLE blocks (
  height INTEGER PRIMARY KEY,
  is_peak BOOLEAN NOT NULL DEFAULT FALSE,
  header_hash BLOB,
  timestamp INTEGER
);

CREATE TABLE coins (
  id INTEGER PRIMARY KEY,
  asset_id INTEGER NOT NULL,
  hash BLOB NOT NULL UNIQUE,
  parent_coin_id BLOB NOT NULL,
  puzzle_hash BLOB NOT NULL,
  amount BLOB NOT NULL,
  is_synced BOOLEAN NOT NULL,
  hint BLOB,
  created_height INTEGER,
  FOREIGN KEY (asset_id) REFERENCES assets(id) ON DELETE CASCADE,
  FOREIGN KEY (created_height) REFERENCES blocks(height) ON DELETE SET NULL
);

CREATE TABLE lineage_proofs (
  id INTEGER PRIMARY KEY,
  coin_id INTEGER NOT NULL UNIQUE,
  parent_parent_coin_id BLOB NOT NULL,
  parent_inner_puzzle_hash BLOB NOT NULL,
  parent_amount BLOB NOT NULL,
  FOREIGN KEY (coin_id) REFERENCES coins(id) ON DELETE CASCADE
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

CREATE TABLE tokens (
  id INTEGER PRIMARY KEY,
  asset_id INTEGER NOT NULL UNIQUE,
  ticker TEXT,
  is_xch BOOLEAN GENERATED ALWAYS AS (asset_id = 0) STORED,
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
  offer_id INTEGER,
  coin_id INTEGER UNIQUE,
  FOREIGN KEY (offer_id) REFERENCES offers(id) ON DELETE CASCADE
  FOREIGN KEY (coin_id) REFERENCES coins(id) ON DELETE CASCADE
  UNIQUE(offer_id, coin_id)
);
