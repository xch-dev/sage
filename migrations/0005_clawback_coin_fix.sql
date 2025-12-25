DROP VIEW clawback_coins;

CREATE VIEW clawback_coins AS
SELECT *
FROM wallet_coins
WHERE 1=1
  AND spent_height IS NULL
  AND clawback_expiration_seconds IS NOT NULL;
