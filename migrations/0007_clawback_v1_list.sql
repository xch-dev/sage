/*
 * V1: list clawback coins for both sender and receiver.
 * Claim eligibility remains gated in UI / claimable_clawback_coins.
 */

DROP VIEW IF EXISTS claimable_clawback_coins;
DROP VIEW clawback_coins;

CREATE VIEW clawback_coins AS
SELECT *
FROM wallet_coins
WHERE 1=1
  AND spent_height IS NULL
  AND (
    (clawback_version = 2 AND clawback_sender_p2_puzzle_id IS NOT NULL)
    OR
    (clawback_version = 1 AND clawback_sender_p2_puzzle_id IS NOT NULL)
    OR
    (clawback_version = 1 AND clawback_receiver_p2_puzzle_id IS NOT NULL)
  );

CREATE VIEW claimable_clawback_coins AS
SELECT *
FROM clawback_coins
WHERE clawback_version = 1
  AND clawback_receiver_p2_puzzle_id IS NOT NULL
  AND created_timestamp IS NOT NULL
  AND unixepoch() >= created_timestamp + clawback_expiration_seconds;
