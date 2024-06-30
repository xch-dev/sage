CREATE TABLE `derivations` (
    `p2_puzzle_hash` BLOB NOT NULL PRIMARY KEY,
    `index` INTEGER NOT NULL,
    `hardened` INTEGER NOT NULL CHECK (`hardened` IN (0, 1)),
    `synthetic_key` BLOB NOT NULL
);

CREATE TABLE `coin_states` (
    `coin_id` BLOB NOT NULL PRIMARY KEY,
    `parent_coin_id` BLOB NOT NULL,
    `puzzle_hash` BLOB NOT NULL,
    `amount` BLOB NOT NULL,
    `spent_height` INTEGER,
    `created_height` INTEGER
);

CREATE TABLE `cat_coins` (
    `coin_id` BLOB NOT NULL PRIMARY KEY,
    `parent_parent_coin_id` BLOB NOT NULL,
    `parent_inner_puzzle_hash` BLOB NOT NULL,
    `parent_amount` BLOB NOT NULL,
    `p2_puzzle_hash` BLOB NOT NULL,
    `asset_id` BLOB NOT NULL,
    FOREIGN KEY (`coin_id`) REFERENCES `coin_states` (`coin_id`)
);
