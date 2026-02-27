/*
 * Vaults are an asset with kind = 4
 */
CREATE TABLE vaults (
    id INTEGER NOT NULL PRIMARY KEY,
    asset_id INTEGER NOT NULL UNIQUE,
    FOREIGN KEY (asset_id) REFERENCES assets(id) ON DELETE CASCADE
);

/*
 * P2 vault is a p2 puzzle with kind = 4
 */
 CREATE TABLE p2_vaults (
  id INTEGER NOT NULL PRIMARY KEY,
  p2_puzzle_id INTEGER NOT NULL UNIQUE,
  vault_asset_id INTEGER NOT NULL,
  FOREIGN KEY (p2_puzzle_id) REFERENCES p2_puzzles(id) ON DELETE CASCADE,
  FOREIGN KEY (vault_asset_id) REFERENCES assets(id) ON DELETE CASCADE
);

/*
 * We're starting with single signer vault support with recovery fro now
 */
CREATE TABLE vault_configs (
    id INTEGER NOT NULL PRIMARY KEY,
    custody_hash BLOB NOT NULL,
    custody_key_id INTEGER NOT NULL,
    recovery_key_id INTEGER NOT NULL,
    recovery_timelock INTEGER NOT NULL,
    FOREIGN KEY (custody_key_id) REFERENCES vault_keys(id),
    FOREIGN KEY (recovery_key_id) REFERENCES vault_keys(id)
);

/*
 * A single key that can be used to sign for a vault.
 * The kind represents the type of key:
 * BLS = 0
 * Secp256r1 = 1
 */
CREATE TABLE vault_keys (
    id INTEGER NOT NULL PRIMARY KEY,
    kind INTEGER NOT NULL,
    public_key BLOB NOT NULL,
    fast_forwardable BOOLEAN NOT NULL,
    CONSTRAINT unique_key UNIQUE (kind, public_key)
);
