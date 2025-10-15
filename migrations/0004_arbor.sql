/*
 * P2 arbor is a p2 puzzle with kind = 3
 */
 CREATE TABLE p2_arbor (
  id INTEGER NOT NULL PRIMARY KEY,
  p2_puzzle_id INTEGER NOT NULL UNIQUE,
  key BLOB NOT NULL,
  FOREIGN KEY (p2_puzzle_id) REFERENCES p2_puzzles(id) ON DELETE CASCADE
);
