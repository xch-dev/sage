CREATE TABLE `peaks` (
    `height` INTEGER NOT NULL PRIMARY KEY,
    `header_hash` BLOB NOT NULL
);

CREATE TABLE `derivations` (
    `p2_puzzle_hash` BLOB NOT NULL PRIMARY KEY,
    `index` INTEGER NOT NULL,
    `hardened` BOOLEAN NOT NULL,
    `synthetic_key` BLOB NOT NULL
);

CREATE INDEX `derivation_index` ON `derivations` (`index`, `hardened`);
CREATE INDEX `derivation_key` ON `derivations` (`synthetic_key`);

CREATE TABLE `coin_states` (
    `coin_id` BLOB NOT NULL PRIMARY KEY,
    `parent_coin_id` BLOB NOT NULL,
    `puzzle_hash` BLOB NOT NULL,
    `amount` BLOB NOT NULL,
    `spent_height` INTEGER,
    `created_height` INTEGER,
    `hint` BLOB,
    `synced` BOOLEAN NOT NULL
);

CREATE INDEX `coin_parent` ON `coin_states` (`parent_coin_id`);
CREATE INDEX `coin_puzzle_hash` ON `coin_states` (`puzzle_hash`);
CREATE INDEX `coin_spent` ON `coin_states` (`spent_height`);
CREATE INDEX `coin_created` ON `coin_states` (`created_height`);
CREATE INDEX `coin_hint` ON `coin_states` (`hint`);
CREATE INDEX `coin_synced` ON `coin_states` (`synced`);

CREATE TABLE `unknown_coins` (
    `coin_id` BLOB NOT NULL PRIMARY KEY,
    FOREIGN KEY (`coin_id`) REFERENCES `coin_states` (`coin_id`)
);

CREATE TABLE `p2_coins` (
    `coin_id` BLOB NOT NULL PRIMARY KEY,
    FOREIGN KEY (`coin_id`) REFERENCES `coin_states` (`coin_id`)
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

CREATE INDEX `cat_p2` ON `cat_coins` (`p2_puzzle_hash`);
CREATE INDEX `cat_asset_id` ON `cat_coins` (`asset_id`);

CREATE TABLE `did_coins` (
    `coin_id` BLOB NOT NULL PRIMARY KEY,
    `parent_parent_coin_id` BLOB NOT NULL,
    `parent_inner_puzzle_hash` BLOB NOT NULL,
    `parent_amount` BLOB NOT NULL,
    `launcher_id` BLOB NOT NULL,
    `recovery_list_hash` BLOB,
    `num_verifications_required` BLOB NOT NULL,
    `metadata` BLOB NOT NULL,
    `p2_puzzle_hash` BLOB NOT NULL,
    FOREIGN KEY (`coin_id`) REFERENCES `coin_states` (`coin_id`)
);

CREATE INDEX `did_launcher_id` ON `did_coins` (`launcher_id`);
CREATE INDEX `did_p2` ON `did_coins` (`p2_puzzle_hash`);

CREATE TABLE `nft_coins` (
    `coin_id` BLOB NOT NULL PRIMARY KEY,
    `parent_parent_coin_id` BLOB NOT NULL,
    `parent_inner_puzzle_hash` BLOB NOT NULL,
    `parent_amount` BLOB NOT NULL,
    `launcher_id` BLOB NOT NULL,
    `metadata` BLOB NOT NULL,
    `metadata_updater_puzzle_hash` BLOB NOT NULL,
    `current_owner` BLOB,
    `royalty_puzzle_hash` BLOB NOT NULL,
    `royalty_ten_thousandths` INTEGER NOT NULL,
    `p2_puzzle_hash` BLOB NOT NULL,
    `data_hash` BLOB,
    `metadata_hash` BLOB,
    `license_hash` BLOB,
    FOREIGN KEY (`coin_id`) REFERENCES `coin_states` (`coin_id`)
);

CREATE INDEX `nft_launcher_id` ON `nft_coins` (`launcher_id`);
CREATE INDEX `nft_p2` ON `nft_coins` (`p2_puzzle_hash`);

CREATE TABLE `nft_data` (
    `hash` BLOB NOT NULL PRIMARY KEY,
    `data` BLOB NOT NULL,
    `mime_type` TEXT NOT NULL
);

CREATE TABLE `nft_uris` (
    `uri` TEXT NOT NULL,
    `hash` BLOB NOT NULL,
    `checked` BOOLEAN NOT NULL,
    PRIMARY KEY (`uri`, `hash`)
);

CREATE INDEX `nft_uri_checked_hash` ON `nft_uris` (`checked`, `hash`);

CREATE TABLE `cats` (
    `asset_id` BLOB NOT NULL PRIMARY KEY,
    `name` TEXT,
    `description` TEXT,
    `ticker` TEXT,
    `precision` INTEGER NOT NULL,
    `icon_url` TEXT
);

