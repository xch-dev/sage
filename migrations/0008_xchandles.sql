/*
 * Handles resolved through XCHandles, keyed by the registry they were resolved against so
 * that mappings from one deployment are never used for another.
 *
 * The resolved puzzle hash and proof columns hold the last value that was verified against
 * the chain. They are kept when a later verification fails, so the warning about a changed
 * mapping is never lost, but such rows can't be used until they're verified again.
 *
 * status: 0 = verified, 1 = unverified (a later lookup was rejected), 2 = stale (needs
 * to be verified again, for example after a reorg)
 *
 * previous_resolved_* is set when a verification finds a different resolved address than
 * the last verified one, and is cleared once the change is acknowledged.
 */
CREATE TABLE xchandles (
  id INTEGER NOT NULL PRIMARY KEY,
  registry_launcher_id BLOB NOT NULL,
  handle TEXT NOT NULL,
  handle_hash BLOB NOT NULL,
  owner_launcher_id BLOB NOT NULL,
  resolved_launcher_id BLOB NOT NULL,
  resolved_puzzle_hash BLOB NOT NULL,
  expiration INTEGER NOT NULL,
  slot_coin_id BLOB NOT NULL,
  slot_counter INTEGER NOT NULL,
  slot_left_handle_hash BLOB NOT NULL,
  slot_right_handle_hash BLOB NOT NULL,
  slot_parent_id BLOB NOT NULL,
  slot_parent_parent_id BLOB NOT NULL,
  slot_parent_inner_puzzle_hash BLOB NOT NULL,
  slot_confirmation_height INTEGER NOT NULL,
  resolved_coin_id BLOB NOT NULL,
  resolved_confirmation_height INTEGER NOT NULL,
  indexed_height INTEGER NOT NULL,
  verified_height INTEGER NOT NULL,
  verified_timestamp INTEGER NOT NULL,
  last_verified_at INTEGER NOT NULL,
  status INTEGER NOT NULL,
  status_reason TEXT,
  previous_resolved_launcher_id BLOB,
  previous_resolved_puzzle_hash BLOB,
  last_sent_puzzle_hash BLOB,
  UNIQUE (registry_launcher_id, handle)
);

CREATE INDEX xchandles_resolved_puzzle_hash ON xchandles (registry_launcher_id, resolved_puzzle_hash);
