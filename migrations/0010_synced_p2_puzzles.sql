/*
 * Whether a p2 puzzle hash has been synced from the start of the chain all the way to the
 * end. Sync only resumes from the last peak for puzzle hashes that have been, so an
 * interrupted sync of a new puzzle hash can't leave a gap in its history.
 *
 * Puzzle hashes from before this were assumed to be synced, so they still are.
 */
ALTER TABLE p2_puzzles ADD COLUMN is_synced BOOLEAN NOT NULL DEFAULT FALSE;

UPDATE p2_puzzles SET is_synced = TRUE;
