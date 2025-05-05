CREATE TABLE `options` (
    `launcher_id` BLOB NOT NULL PRIMARY KEY,
    `coin_id` BLOB NOT NULL,
    `visible` BOOLEAN NOT NULL,
    `is_owned` BOOLEAN NOT NULL,
    `created_height` INTEGER,
    `is_pending` BOOLEAN GENERATED ALWAYS AS (`created_height` IS NULL) STORED,
    FOREIGN KEY (`coin_id`) REFERENCES `nft_coins` (`coin_id`) ON DELETE CASCADE
);

CREATE INDEX `option_coin_id` ON `nfts` (`coin_id`);
CREATE INDEX `option_recent` ON `nfts` (`is_owned`, `visible` DESC, `is_pending` DESC, `created_height` DESC, `launcher_id` ASC);

CREATE TABLE `option_coins` (
    `coin_id` BLOB NOT NULL PRIMARY KEY,
    `parent_parent_coin_id` BLOB NOT NULL,
    `parent_inner_puzzle_hash` BLOB NOT NULL,
    `parent_amount` BLOB NOT NULL,
    `launcher_id` BLOB NOT NULL,
    `underlying_coin_id` BLOB NOT NULL,
    `underlying_delegated_puzzle_hash` BLOB NOT NULL,
    `p2_puzzle_hash` BLOB NOT NULL,
    FOREIGN KEY (`coin_id`) REFERENCES `coin_states` (`coin_id`) ON DELETE CASCADE
);

CREATE INDEX `option_launcher_id` ON `option_coins` (`launcher_id`);
