/*
 * Options are an asset with kind = 3
 */
CREATE TABLE options (
    id INTEGER NOT NULL PRIMARY KEY,
    asset_id INTEGER NOT NULL UNIQUE,
    underlying_coin_id INTEGER NOT NULL,
    underlying_delegated_puzzle_hash BLOB NOT NULL,
    strike_asset_id INTEGER NOT NULL,
    strike_amount BLOB NOT NULL,
    FOREIGN KEY (asset_id) REFERENCES assets(id) ON DELETE CASCADE,
    FOREIGN KEY (underlying_coin_id) REFERENCES coins(id) ON DELETE CASCADE,
    FOREIGN KEY (strike_asset_id) REFERENCES assets(id) ON DELETE CASCADE
);

/*
 * P2 options are a p2 puzzle with kind = 2
 */
CREATE TABLE p2_options (
    id INTEGER NOT NULL PRIMARY KEY,
    p2_puzzle_id INTEGER NOT NULL UNIQUE,
    option_asset_id INTEGER NOT NULL,
    creator_puzzle_hash BLOB NOT NULL,
    expiration_seconds INTEGER NOT NULL,
    FOREIGN KEY (p2_puzzle_id) REFERENCES p2_puzzles(id) ON DELETE CASCADE,
    FOREIGN KEY (option_asset_id) REFERENCES assets(id) ON DELETE CASCADE
);

CREATE INDEX idx_options_asset_id ON options(asset_id);
CREATE INDEX idx_options_underlying_coin_id ON options(underlying_coin_id);
CREATE INDEX idx_options_strike_asset_id ON options(strike_asset_id);
CREATE INDEX idx_p2_options_p2_puzzle_id ON p2_options(p2_puzzle_id);
CREATE INDEX idx_p2_options_option_asset_id ON p2_options(option_asset_id);

DROP VIEW wallet_coins;

CREATE VIEW wallet_coins AS
SELECT
  coins.id AS coin_id,
  coins.hash AS coin_hash,
  coins.asset_id AS asset_id,
  coins.parent_coin_hash,
  coins.puzzle_hash,
  coins.amount,
  coins.p2_puzzle_id,
  coins.spent_height,
  coins.created_height,
  assets.hash AS asset_hash,
  assets.kind AS asset_kind,
  assets.name AS asset_name,
  assets.ticker AS asset_ticker,
  assets.precision AS asset_precision,
  assets.icon_url AS asset_icon_url,
  assets.description AS asset_description,
  assets.is_visible AS asset_is_visible,
  assets.is_sensitive_content AS asset_is_sensitive_content,
  assets.hidden_puzzle_hash AS asset_hidden_puzzle_hash,
  p2_puzzles.hash AS p2_puzzle_hash,
  p2_puzzles.kind AS p2_puzzle_kind,
  clawbacks.sender_puzzle_hash AS clawback_sender_puzzle_hash,
  clawbacks.receiver_puzzle_hash AS clawback_receiver_puzzle_hash,
  clawbacks.expiration_seconds AS clawback_expiration_seconds,
  p2_options.option_asset_id AS option_asset_id,
  p2_options.creator_puzzle_hash AS option_creator_puzzle_hash,
  p2_options.expiration_seconds AS option_expiration_seconds,
  sender_p2_puzzle.id AS clawback_sender_p2_puzzle_id,
  receiver_p2_puzzle.id AS clawback_receiver_p2_puzzle_id,
  created_blocks.timestamp AS created_timestamp,
  spent_blocks.timestamp AS spent_timestamp,
  (
    SELECT hash FROM mempool_items
    INNER JOIN mempool_coins ON mempool_coins.mempool_item_id = mempool_items.id
    WHERE mempool_coins.coin_id = coins.id
    AND mempool_coins.is_input = TRUE
    LIMIT 1
  ) AS mempool_item_hash,
  (
    SELECT hash FROM offers
    INNER JOIN offer_coins ON offer_coins.offer_id = offers.id
    WHERE offer_coins.coin_id = coins.id
    AND offers.status <= 1
    LIMIT 1
  ) AS offer_hash
FROM coins
  INNER JOIN assets ON assets.id = coins.asset_id
  INNER JOIN p2_puzzles ON p2_puzzles.id = coins.p2_puzzle_id
  LEFT JOIN clawbacks ON clawbacks.p2_puzzle_id = p2_puzzles.id
  LEFT JOIN p2_options ON p2_options.p2_puzzle_id = p2_puzzles.id
  LEFT JOIN p2_puzzles AS sender_p2_puzzle ON sender_p2_puzzle.hash = clawbacks.sender_puzzle_hash
  LEFT JOIN p2_puzzles AS receiver_p2_puzzle ON receiver_p2_puzzle.hash = clawbacks.receiver_puzzle_hash
  LEFT JOIN blocks AS created_blocks ON created_blocks.height = coins.created_height
  LEFT JOIN blocks AS spent_blocks ON spent_blocks.height = coins.spent_height;
