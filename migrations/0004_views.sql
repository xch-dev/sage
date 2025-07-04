CREATE VIEW spendable_coins AS
SELECT
  coins.id AS coin_id,
  coins.hash AS coin_hash,
  coins.asset_id AS asset_id,
  coins.parent_coin_hash,
  coins.puzzle_hash,
  coins.amount,
  coins.hidden_puzzle_hash,
  coins.p2_puzzle_id,
  assets.hash AS asset_hash,
  assets.kind AS asset_kind,
  p2_puzzles.hash AS p2_puzzle_hash
FROM coins
  INNER JOIN assets ON assets.id = coins.asset_id
  INNER JOIN p2_puzzles ON p2_puzzles.id = coins.p2_puzzle_id
  LEFT JOIN mempool_coins ON mempool_coins.coin_id = coins.id
WHERE 1=1
  AND spent_height IS NULL
  AND mempool_coins.id IS NULL
  AND NOT EXISTS (
    SELECT 1 FROM offer_coins
    INNER JOIN offers ON offers.id = offer_coins.offer_id
    WHERE offer_coins.coin_id = coins.id
    AND offers.status <= 1
  )
  AND NOT EXISTS (
    SELECT 1 FROM clawbacks
    WHERE clawbacks.p2_puzzle_id = p2_puzzles.id
    AND (
      NOT EXISTS (SELECT 1 FROM p2_puzzles WHERE p2_puzzles.hash = clawbacks.receiver_puzzle_hash)
      OR unixepoch() < clawbacks.expiration_seconds
    )
  )
  AND NOT EXISTS (
    SELECT 1 FROM p2_options
    LEFT JOIN options ON options.asset_id = p2_options.option_asset_id
    WHERE p2_options.p2_puzzle_id = p2_puzzles.id
    AND (
      options.id IS NULL
      OR NOT EXISTS (SELECT 1 FROM p2_puzzles WHERE p2_puzzles.hash = options.creator_puzzle_hash)
      OR unixepoch() < options.expiration_seconds
    )
  );

CREATE VIEW owned_coins AS
SELECT
  coins.id AS coin_id,
  coins.hash AS coin_hash,
  coins.asset_id AS asset_id,
  coins.parent_coin_hash,
  coins.puzzle_hash,
  coins.amount,
  coins.hidden_puzzle_hash,
  coins.p2_puzzle_id,
  assets.hash AS asset_hash,
  assets.kind AS asset_kind,
  p2_puzzles.hash AS p2_puzzle_hash
FROM coins
  INNER JOIN assets ON assets.id = coins.asset_id
  INNER JOIN p2_puzzles ON p2_puzzles.id = coins.p2_puzzle_id
  LEFT JOIN mempool_coins ON mempool_coins.coin_id = coins.id
WHERE 1=1
  AND spent_height IS NULL
  AND (mempool_coins.id IS NULL OR mempool_coins.is_input = FALSE);

CREATE VIEW transaction_coins AS
SELECT
  blocks.height,
  blocks.timestamp,
  coins.hash AS coin_id,
  coins.puzzle_hash,
  coins.parent_coin_hash,
  coins.amount,
  coins.created_height = blocks.height AS is_created_in_block,
  coins.spent_height = blocks.height AS is_spent_in_block,
  p2_puzzles.hash AS p2_puzzle_hash,
  assets.hash AS asset_hash,
  assets.name AS asset_name,
  assets.icon_url AS asset_icon_url,
  assets.kind AS asset_kind,
  assets.description AS asset_description,
  assets.is_visible AS asset_is_visible,
  assets.is_sensitive_content AS asset_is_sensitive_content,
  assets.created_height AS asset_created_height,
  tokens.ticker
FROM blocks
LEFT JOIN coins ON coins.created_height = blocks.height OR coins.spent_height = blocks.height
INNER JOIN assets ON assets.id = coins.asset_id
LEFT JOIN p2_puzzles ON p2_puzzles.id = coins.p2_puzzle_id
LEFT JOIN tokens ON tokens.asset_id = assets.id
