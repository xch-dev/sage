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
  p2_puzzles.hash AS p2_puzzle_hash
FROM coins
  INNER JOIN assets ON assets.id = coins.asset_id
  INNER JOIN p2_puzzles ON p2_puzzles.id = coins.p2_puzzle_id
  LEFT JOIN mempool_coins ON mempool_coins.coin_id = coins.id
WHERE 1=1
  AND spent_height IS NULL
  AND mempool_coins.id IS NULL
  AND p2_puzzles.kind IN (0, 1)
  AND NOT EXISTS (
    SELECT 1 FROM offer_coins
    INNER JOIN offers ON offers.id = offer_coins.offer_id
    WHERE offer_coins.coin_id = coins.id
    AND offers.status <= 1
  )
  AND NOT EXISTS (
    SELECT 1 FROM clawbacks -- If it's not a clawback, it's spendable
    WHERE clawbacks.p2_puzzle_id = p2_puzzles.id
    AND (
      -- If it is a clawback, it's not spendable if the receiver puzzle hash isn't ours
      -- Technically, we can spend it if we're the sender, but we don't want to select it or include it in the spendable balance
      NOT EXISTS (SELECT 1 FROM p2_puzzles WHERE p2_puzzles.hash = receiver_puzzle_hash)
      -- It's not spendable if the clawback hasn't expired yet
      OR unixepoch() < expiration_seconds
    )
  );

CREATE VIEW wallet_coins AS
SELECT
  coins.id AS coin_id,
  coins.hash AS coin_hash,
  coins.asset_id AS asset_id,
  coins.parent_coin_hash,
  coins.puzzle_hash,
  coins.amount,
  coins.hidden_puzzle_hash,
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
  p2_puzzles.hash AS p2_puzzle_hash,
  clawbacks.sender_puzzle_hash AS clawback_sender_puzzle_hash,
  clawbacks.receiver_puzzle_hash AS clawback_receiver_puzzle_hash,
  clawbacks.expiration_seconds AS clawback_expiration_seconds,
  sender_p2_puzzle.id AS clawback_sender_p2_puzzle_id,
  receiver_p2_puzzle.id AS clawback_receiver_p2_puzzle_id
FROM coins
  INNER JOIN assets ON assets.id = coins.asset_id
  INNER JOIN p2_puzzles ON p2_puzzles.id = coins.p2_puzzle_id
  LEFT JOIN clawbacks ON clawbacks.p2_puzzle_id = p2_puzzles.id
  LEFT JOIN p2_puzzles AS sender_p2_puzzle ON sender_p2_puzzle.hash = clawbacks.sender_puzzle_hash
  LEFT JOIN p2_puzzles AS receiver_p2_puzzle ON receiver_p2_puzzle.hash = clawbacks.receiver_puzzle_hash;

CREATE VIEW owned_coins AS
SELECT *
FROM wallet_coins
WHERE 1=1
  AND spent_height IS NULL
  AND NOT EXISTS (
    SELECT 1 FROM mempool_coins
    WHERE mempool_coins.coin_id = wallet_coins.coin_id AND mempool_coins.is_input = TRUE
  )
  AND (
    clawback_expiration_seconds IS NULL
    OR clawback_receiver_p2_puzzle_id IS NOT NULL
    OR (clawback_sender_p2_puzzle_id IS NOT NULL AND unixepoch() < clawback_expiration_seconds)
  );

CREATE VIEW spent_coins AS
SELECT *
FROM wallet_coins
WHERE spent_height IS NOT NULL
  OR EXISTS (
    SELECT 1 FROM mempool_coins
    WHERE mempool_coins.coin_id = wallet_coins.coin_id AND mempool_coins.is_input = TRUE
  )
  OR (
    clawback_expiration_seconds IS NOT NULL
    AND clawback_receiver_p2_puzzle_id IS NULL
    AND clawback_sender_p2_puzzle_id IS NOT NULL
    AND unixepoch() >= clawback_expiration_seconds
  );

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
  assets.ticker AS asset_ticker,
  assets.precision AS asset_precision,
  assets.icon_url AS asset_icon_url,
  assets.kind AS asset_kind,
  assets.description AS asset_description,
  assets.is_visible AS asset_is_visible,
  assets.is_sensitive_content AS asset_is_sensitive_content
FROM blocks
LEFT JOIN coins ON coins.created_height = blocks.height OR coins.spent_height = blocks.height
INNER JOIN assets ON assets.id = coins.asset_id
LEFT JOIN p2_puzzles ON p2_puzzles.id = coins.p2_puzzle_id;

CREATE VIEW owned_nfts AS
  SELECT        
      asset_hash, asset_name, asset_ticker, asset_precision, asset_icon_url,
      asset_description, asset_is_sensitive_content, asset_is_visible,
      nfts.minter_hash, owner_hash, metadata, metadata_updater_puzzle_hash,
      royalty_puzzle_hash, royalty_basis_points, data_hash, metadata_hash, license_hash,
      edition_number, edition_total, nfts.collection_id,
      parent_coin_hash, puzzle_hash, amount, p2_puzzle_hash, created_height, spent_height,
      (
          SELECT hash FROM offers
          INNER JOIN offer_coins ON offer_coins.offer_id = offers.id
          WHERE offer_coins.coin_id = owned_coins.coin_id
          AND offers.status <= 1
          LIMIT 1
      ) AS offer_hash,
      (
          SELECT timestamp FROM blocks
          WHERE height = owned_coins.created_height
      ) AS created_timestamp,
      (
          SELECT timestamp FROM blocks
          WHERE height = owned_coins.spent_height
      ) AS spent_timestamp
  FROM owned_coins
  INNER JOIN nfts ON nfts.asset_id = owned_coins.asset_id
