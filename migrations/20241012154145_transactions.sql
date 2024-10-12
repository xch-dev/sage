CREATE TABLE `transactions` (
    `transaction_id` BLOB NOT NULL PRIMARY KEY,
    `aggregated_signature` BLOB NOT NULL,
    `fee` BLOB NOT NULL,
    `submitted_at` INTEGER,
    `expiration_height` INTEGER
);

CREATE TABLE `transaction_spends` (
    `coin_id` BLOB NOT NULL PRIMARY KEY,
    `transaction_id` BLOB NOT NULL,
    `parent_coin_id` BLOB NOT NULL,
    `puzzle_hash` BLOB NOT NULL,
    `amount` BLOB NOT NULL,
    `puzzle_reveal` BLOB NOT NULL,
    `solution` BLOB NOT NULL,
    FOREIGN KEY (`transaction_id`) REFERENCES `transactions` (`transaction_id`) ON DELETE CASCADE
);
