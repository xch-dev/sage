-- Transaction history table to persist confirmed transaction data
-- This allows webhook consumers to look up transaction details by transaction_id
-- after the mempool item has been deleted

CREATE TABLE transaction_history (
  id INTEGER NOT NULL PRIMARY KEY,
  hash BLOB NOT NULL UNIQUE,
  height INTEGER NOT NULL,
  fee BLOB NOT NULL,
  confirmed_timestamp INTEGER NOT NULL
);

CREATE TABLE transaction_history_coins (
  id INTEGER NOT NULL PRIMARY KEY,
  transaction_history_id INTEGER NOT NULL,
  coin_id BLOB NOT NULL,
  is_input BOOLEAN NOT NULL,
  FOREIGN KEY (transaction_history_id) REFERENCES transaction_history(id) ON DELETE CASCADE
);

CREATE INDEX idx_transaction_history_height ON transaction_history(height);
CREATE INDEX idx_transaction_history_coins_tx ON transaction_history_coins(transaction_history_id);
