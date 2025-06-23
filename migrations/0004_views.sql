CREATE VIEW spendable_coins AS
SELECT
  coins.id,
  coins.hash,
  coins.asset_id,
  coins.parent_coin_hash,
  coins.puzzle_hash,
  coins.amount
FROM coins
  INNER JOIN assets ON assets.id = coins.asset_id
  LEFT JOIN offer_coins ON offer_coins.coin_id = coins.id
  LEFT JOIN offers ON offers.id = offer_coins.offer_id
  LEFT JOIN transaction_coins ON transaction_coins.coin_id = coins.id
  LEFT JOIN public_keys ON public_keys.p2_puzzle_id = coins.p2_puzzle_id
  LEFT JOIN clawbacks ON clawbacks.p2_puzzle_id = coins.p2_puzzle_id
  LEFT JOIN p2_options ON p2_options.p2_puzzle_id = coins.p2_puzzle_id
WHERE 1=1
  AND spent_height IS NULL
  AND assets.kind = 0
  AND (offers.id IS NULL OR offers.status > 1)
  AND transaction_coins.id IS NULL
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
  coins.amount 
FROM coins
  INNER JOIN assets ON asset_id = assets.id
  LEFT JOIN transaction_coins ON transaction_coins.coin_id = coins.id
  LEFT JOIN transaction_spends ON transaction_spends.transaction_coin_id = transaction_coins.id
WHERE 1=1
  AND spent_height IS NULL
  AND assets.kind = 0
  AND transaction_spends.id IS NULL;