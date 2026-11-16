/*
 * A vault recovery that was started on chain, keyed by the custody hash of the vault in its
 * recovery state. It's recorded from the spend that started it, so a recovery is noticed even if
 * its memo was left out, and can be cancelled with the custody from before it started.
 *
 * custody_member_hash is the left side of the recovery state's 1 of 2, which is the custody that can
 * cancel the recovery. recovery_member_hash is the right side, which finishes it after
 * recovery_timelock. finish_puzzle_hash is the puzzle hash of the timelocked member that finishes
 * the recovery, which the spend that started it revealed.
 *
 * is_started_locally is set when this wallet started the recovery, so the user isn't alerted about
 * their own recovery. A recovery that the wallet didn't start is the reason for the timelock, so the
 * user is alerted until it's cancelled or finished.
 */
CREATE TABLE vault_recoveries (
  id INTEGER NOT NULL PRIMARY KEY,
  custody_hash BLOB NOT NULL UNIQUE,
  custody_member_hash BLOB NOT NULL,
  recovery_member_hash BLOB NOT NULL,
  finish_puzzle_hash BLOB NOT NULL,
  recovery_timelock INTEGER NOT NULL,
  is_started_locally BOOLEAN NOT NULL DEFAULT FALSE
);
