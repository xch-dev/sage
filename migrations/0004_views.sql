CREATE VIEW spendable_coins AS
SELECT
  coins.id,
  coins.hash,
  coins.asset_id,
  coins.parent_coin_hash,
  coins.puzzle_hash,
  coins.amount,
  coins.hidden_puzzle_hash,
  coins.p2_puzzle_id
FROM coins
  INNER JOIN assets ON assets.id = coins.asset_id
  LEFT JOIN offer_coins ON offer_coins.coin_id = coins.id
  LEFT JOIN offers ON offers.id = offer_coins.offer_id
  LEFT JOIN mempool_coins ON mempool_coins.coin_id = coins.id
  LEFT JOIN public_keys ON public_keys.p2_puzzle_id = coins.p2_puzzle_id
  LEFT JOIN clawbacks ON clawbacks.p2_puzzle_id = coins.p2_puzzle_id
  LEFT JOIN p2_options ON p2_options.p2_puzzle_id = coins.p2_puzzle_id
WHERE 1=1
  AND spent_height IS NULL
  AND assets.kind = 0
  AND (offers.id IS NULL OR offers.status > 1)
  AND mempool_coins.id IS NULL
  AND (
    clawbacks.id IS NULL
    OR EXISTS (
        SELECT 1 FROM p2_puzzles 
        WHERE p2_puzzles.hash = clawbacks.receiver_puzzle_hash
        AND unixepoch() >= clawbacks.expiration_seconds
    )
  )
  AND (
    p2_options.id IS NULL
    OR EXISTS (
        SELECT 1 FROM options
        INNER JOIN p2_puzzles ON p2_puzzles.hash = options.creator_puzzle_hash
        WHERE options.asset_id = p2_options.option_asset_id
        AND unixepoch() >= options.expiration_seconds
    )
  );



CREATE VIEW owned_coins AS
SELECT
  coins.id,
  coins.hash,
  coins.asset_id,
  coins.parent_coin_hash,
  coins.puzzle_hash,
  coins.amount,
  coins.hidden_puzzle_hash,
  coins.p2_puzzle_id
FROM coins
  INNER JOIN assets ON assets.id = coins.asset_id
  LEFT JOIN mempool_coins ON mempool_coins.coin_id = coins.id
WHERE 1=1
  AND spent_height IS NULL
  AND assets.kind = 0
  AND NOT mempool_coins.is_input;

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
	assets.hash AS asset_hash,
	assets.name,
	assets.icon_url,
	assets.kind,
  assets.description,
  assets.is_visible,
  assets.is_sensitive_content,
  assets.created_height,
	p2_puzzles.hash AS p2_puzzle_hash
FROM blocks
LEFT JOIN coins ON coins.created_height = blocks.height OR coins.spent_height = blocks.height
INNER JOIN assets ON assets.id = coins.asset_id
LEFT JOIN p2_puzzles ON p2_puzzles.id = coins.p2_puzzle_id