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
    `synced` BOOLEAN NOT NULL,
    `transaction_id` BLOB,
    FOREIGN KEY (`transaction_id`) REFERENCES `transactions` (`transaction_id`) ON DELETE CASCADE
);

CREATE INDEX `coin_puzzle_hash` ON `coin_states` (`puzzle_hash`);
CREATE INDEX `coin_hint` ON `coin_states` (`hint`);
CREATE INDEX `coin_spent` ON `coin_states` (`spent_height`);
CREATE INDEX `coin_created` ON `coin_states` (`created_height`);
CREATE INDEX `coin_synced` ON `coin_states` (`synced`);
CREATE INDEX `coin_height` ON `coin_states` (`spent_height` ASC, `created_height` DESC);
CREATE INDEX `coin_transaction` ON `coin_states` (`transaction_id`);

CREATE TABLE `transactions` (
    `transaction_id` BLOB NOT NULL PRIMARY KEY,
    `aggregated_signature` BLOB NOT NULL,
    `fee` BLOB NOT NULL,
    `submitted_at` INTEGER
);

CREATE TABLE `transaction_spends` (
    `coin_id` BLOB NOT NULL PRIMARY KEY,
    `index` INTEGER NOT NULL,
    `transaction_id` BLOB NOT NULL,
    `parent_coin_id` BLOB NOT NULL,
    `puzzle_hash` BLOB NOT NULL,
    `amount` BLOB NOT NULL,
    `puzzle_reveal` BLOB NOT NULL,
    `solution` BLOB NOT NULL,
    FOREIGN KEY (`transaction_id`) REFERENCES `transactions` (`transaction_id`) ON DELETE CASCADE
);

CREATE INDEX `indexed_spend` ON `transaction_spends` (`transaction_id`, `index` ASC);

CREATE TABLE `unknown_coins` (
    `coin_id` BLOB NOT NULL PRIMARY KEY,
    FOREIGN KEY (`coin_id`) REFERENCES `coin_states` (`coin_id`) ON DELETE CASCADE
);

CREATE TABLE `p2_coins` (
    `coin_id` BLOB NOT NULL PRIMARY KEY,
    FOREIGN KEY (`coin_id`) REFERENCES `coin_states` (`coin_id`) ON DELETE CASCADE
);

CREATE TABLE `cats` (
    `asset_id` BLOB NOT NULL PRIMARY KEY,
    `name` TEXT,
    `ticker` TEXT,
    `description` TEXT,
    `icon_url` TEXT,
    `visible` BOOLEAN NOT NULL
);

CREATE TABLE `cat_coins` (
    `coin_id` BLOB NOT NULL PRIMARY KEY,
    `parent_parent_coin_id` BLOB NOT NULL,
    `parent_inner_puzzle_hash` BLOB NOT NULL,
    `parent_amount` BLOB NOT NULL,
    `p2_puzzle_hash` BLOB NOT NULL,
    `asset_id` BLOB NOT NULL,
    FOREIGN KEY (`coin_id`) REFERENCES `coin_states` (`coin_id`) ON DELETE CASCADE
);

CREATE INDEX `cat_p2` ON `cat_coins` (`p2_puzzle_hash`);
CREATE INDEX `cat_asset_id` ON `cat_coins` (`asset_id`);

CREATE TABLE `dids` (
    `launcher_id` BLOB NOT NULL PRIMARY KEY,
    `name` TEXT,
    `visible` BOOLEAN NOT NULL,
    `is_named` BOOLEAN GENERATED ALWAYS AS (`name` IS NOT NULL) STORED
);

CREATE INDEX `did_index` ON `dids` (`visible` DESC, `is_named` DESC, `name` ASC);

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
    FOREIGN KEY (`coin_id`) REFERENCES `coin_states` (`coin_id`) ON DELETE CASCADE
);

CREATE INDEX `did_launcher_id` ON `did_coins` (`launcher_id`);
CREATE INDEX `did_p2` ON `did_coins` (`p2_puzzle_hash`);

CREATE TABLE `nft_collections` (
    `collection_id` BLOB NOT NULL PRIMARY KEY,
    `did_id` BLOB NOT NULL,
    `metadata_collection_id` TEXT NOT NULL,
    `visible` BOOLEAN NOT NULL,
    `icon` TEXT,
    `name` TEXT,
    `is_named` BOOLEAN GENERATED ALWAYS AS (`name` IS NOT NULL) STORED
);

CREATE INDEX `col_named` ON `nft_collections` (`visible` DESC, `is_named` DESC, `name` ASC, `collection_id` ASC);

CREATE TABLE `nfts` (
    `launcher_id` BLOB NOT NULL PRIMARY KEY,
    `coin_id` BLOB NOT NULL,
    `collection_id` BLOB,
    `minter_did` BLOB,
    `owner_did` BLOB,
    `visible` BOOLEAN NOT NULL,
    `sensitive_content` BOOLEAN NOT NULL,
    `name` TEXT,
    `is_named` BOOLEAN GENERATED ALWAYS AS (`name` IS NOT NULL) STORED,
    `created_height` INTEGER,
    `is_pending` BOOLEAN GENERATED ALWAYS AS (`created_height` IS NULL) STORED,
    `metadata_hash` BLOB,
    FOREIGN KEY (`coin_id`) REFERENCES `nft_coins` (`coin_id`) ON DELETE CASCADE
);

CREATE INDEX `nft_metadata` ON `nfts` (`metadata_hash`);
CREATE INDEX `nft_named` ON `nfts` (`visible` DESC, `is_named` DESC, `name` ASC, `launcher_id` ASC);
CREATE INDEX `nft_recent` ON `nfts` (`visible` DESC, `is_pending` DESC, `created_height` DESC, `launcher_id` ASC);

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
    FOREIGN KEY (`coin_id`) REFERENCES `coin_states` (`coin_id`) ON DELETE CASCADE
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
