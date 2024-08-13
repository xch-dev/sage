CREATE TABLE `peaks` (
    `height` INTEGER NOT NULL PRIMARY KEY,
    `header_hash` BLOB NOT NULL
);

CREATE TABLE `derivations` (
    `p2_puzzle_hash` BLOB NOT NULL PRIMARY KEY,
    `index` INTEGER NOT NULL,
    `hardened` INTEGER NOT NULL CHECK (`hardened` IN (0, 1)),
    `synthetic_key` BLOB NOT NULL
);

CREATE INDEX `index_index` ON `derivations` (`index`, `hardened`);
CREATE INDEX `synthetic_key_index` ON `derivations` (`synthetic_key`);

CREATE TABLE `coin_states` (
    `coin_id` BLOB NOT NULL PRIMARY KEY,
    `parent_coin_id` BLOB NOT NULL,
    `puzzle_hash` BLOB NOT NULL,
    `amount` BLOB NOT NULL,
    `spent_height` INTEGER,
    `created_height` INTEGER,
    `synced` INTEGER NOT NULL CHECK (`synced` IN (0, 1))
);

CREATE INDEX `parent_index` ON `coin_states` (`parent_coin_id`);
CREATE INDEX `puzzle_hash_index` ON `coin_states` (`puzzle_hash`);
CREATE INDEX `spent_height_index` ON `coin_states` (`spent_height`);
CREATE INDEX `created_height_index` ON `coin_states` (`created_height`);

CREATE TABLE `cat_coins` (
    `coin_id` BLOB NOT NULL PRIMARY KEY,
    `parent_parent_coin_id` BLOB NOT NULL,
    `parent_inner_puzzle_hash` BLOB NOT NULL,
    `parent_amount` BLOB NOT NULL,
    `p2_puzzle_hash` BLOB NOT NULL,
    `asset_id` BLOB NOT NULL,
    FOREIGN KEY (`coin_id`) REFERENCES `coin_states` (`coin_id`)
);

CREATE INDEX `p2_puzzle_hash_index` ON `cat_coins` (`p2_puzzle_hash`);
CREATE INDEX `asset_id_index` ON `cat_coins` (`asset_id`);
