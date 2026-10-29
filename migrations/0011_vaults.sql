/*
 * Vaults are an asset with kind = 4, whose hash is the launcher id.
 *
 * vault_coins are the coins of a vault singleton's lineage, from the eve coin to the current one.
 * They belong to the vault's asset, and have no p2 puzzle, so they're never part of a balance. The
 * launcher coin belongs to the vault's asset too, so that its children are synced, but has no row
 * here.
 *
 * custody_hash is the singleton's inner puzzle hash. It's NULL for an eve coin whose launcher
 * didn't reveal a memo, until the coin is spent and reveals its inner puzzle. The vault's state is
 * derived from its lineage, so that it follows the coins when a reorg undoes a spend, and its
 * layout is the vault_configs row with the same custody_hash, if one is known.
 */
CREATE TABLE vault_coins (
  id INTEGER NOT NULL PRIMARY KEY,
  coin_id INTEGER NOT NULL UNIQUE,
  custody_hash BLOB,
  FOREIGN KEY (coin_id) REFERENCES coins(id) ON DELETE CASCADE
);

CREATE INDEX idx_vault_coins_custody_hash ON vault_coins(custody_hash);

/*
 * A vault layout, which rebuilds the singleton inner puzzle hash custody_hash. The custody hash
 * doesn't depend on the launcher id, so a config can be shared by vaults.
 *
 * Puzzle versions:
 * Legacy = 0
 * ForceSingletonRecreation = 1
 *
 * Sources:
 * Memo = 0 (decoded from a CHIP-0043 memo, and verified against the custody hash)
 * ConfigFile = 1
 * Unknown = 2
 *
 * A config in the recovery state has no recovery members or threshold. Instead, it has the custody
 * hash and amount that the vault is recreated with once recovery finishes.
 */
CREATE TABLE vault_configs (
  id INTEGER NOT NULL PRIMARY KEY,
  custody_hash BLOB NOT NULL UNIQUE,
  puzzle_version INTEGER NOT NULL,
  custody_threshold INTEGER NOT NULL,
  recovery_threshold INTEGER,
  recovery_timelock INTEGER NOT NULL,
  recovery_custody_hash BLOB,
  recovery_amount BLOB,
  source INTEGER NOT NULL
);

/*
 * A custody or recovery member of a vault config.
 *
 * Kinds:
 * Bls = 0 (key is a G1 public key)
 * Secp256r1 = 1 (key is a compressed public key)
 * Secp256k1 = 2 (key is a compressed public key)
 * Passkey = 3 (key is a compressed secp256r1 public key)
 * Singleton = 4 (key is a launcher id)
 * Custom = 5 (key is the member puzzle hash)
 */
CREATE TABLE vault_config_members (
  id INTEGER NOT NULL PRIMARY KEY,
  vault_config_id INTEGER NOT NULL,
  is_recovery BOOLEAN NOT NULL,
  kind INTEGER NOT NULL,
  key BLOB NOT NULL,
  is_fast_forward BOOLEAN NOT NULL,
  FOREIGN KEY (vault_config_id) REFERENCES vault_configs(id) ON DELETE CASCADE,
  UNIQUE (vault_config_id, is_recovery, kind, key)
);

CREATE INDEX idx_vault_config_members_vault_config_id ON vault_config_members(vault_config_id);

/*
 * P2 singletons are a p2 puzzle with kind = 5. They're the addresses of a vault, which only depend
 * on its launcher id and the nonce, and are spent by a message from the vault.
 */
CREATE TABLE p2_singletons (
  id INTEGER NOT NULL PRIMARY KEY,
  p2_puzzle_id INTEGER NOT NULL UNIQUE,
  launcher_id BLOB NOT NULL,
  nonce INTEGER NOT NULL,
  FOREIGN KEY (p2_puzzle_id) REFERENCES p2_puzzles(id) ON DELETE CASCADE,
  UNIQUE (launcher_id, nonce)
);

/*
 * P2 conditions or singletons are a p2 puzzle with kind = 6. They can be spent by the vault, or
 * with a fixed set of conditions (such as an offer's) without a signature.
 *
 * The version and conditions are only known once the fixed conditions are revealed.
 *
 * Versions:
 * Legacy = 0
 * FixedPuzzleMember = 1
 */
CREATE TABLE p2_conditions_or_singletons (
  id INTEGER NOT NULL PRIMARY KEY,
  p2_puzzle_id INTEGER NOT NULL UNIQUE,
  launcher_id BLOB NOT NULL,
  nonce INTEGER NOT NULL,
  fixed_conditions_hash BLOB NOT NULL,
  version INTEGER,
  conditions BLOB,
  FOREIGN KEY (p2_puzzle_id) REFERENCES p2_puzzles(id) ON DELETE CASCADE
);

CREATE INDEX idx_p2_conditions_or_singletons_launcher_id ON p2_conditions_or_singletons(launcher_id);

