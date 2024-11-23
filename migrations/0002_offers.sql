CREATE TABLE `offers` (
    `offer_id` BLOB NOT NULL PRIMARY KEY,
    `encoded_offer` TEXT NOT NULL,
    `expiration_height` INTEGER,
    `expiration_timestamp` BLOB,
    `fee` BLOB NOT NULL,
    `status` INTEGER NOT NULL,
    `inserted_timestamp` BLOB NOT NULL
);

CREATE INDEX `offer_status` ON `offers` (`status`);
CREATE INDEX `offer_timestamp` ON `offers` (`inserted_timestamp` DESC);

CREATE TABLE `offered_coins` (
    `offer_id` BLOB NOT NULL,
    `coin_id` BLOB NOT NULL,
    PRIMARY KEY (`offer_id`, `coin_id`),
    FOREIGN KEY (`offer_id`) REFERENCES `offers`(`offer_id`) ON DELETE CASCADE
);

CREATE INDEX `offer_coin_id` ON `offered_coins` (`coin_id`);

CREATE TABLE `offer_xch` (
    `offer_id` BLOB NOT NULL,
    `requested` BOOLEAN NOT NULL,
    `amount` BLOB NOT NULL,
    `royalty` BLOB NOT NULL,
    PRIMARY KEY (`offer_id`, `requested`),
    FOREIGN KEY (`offer_id`) REFERENCES `offers`(`offer_id`) ON DELETE CASCADE
);

CREATE INDEX `xch_offer_id` ON `offer_xch` (`offer_id`);

CREATE TABLE `offer_nfts` (
    `offer_id` BLOB NOT NULL,
    `requested` BOOLEAN NOT NULL,
    `launcher_id` BLOB NOT NULL,
    `royalty_puzzle_hash` BLOB NOT NULL,
    `royalty_ten_thousandths` INTEGER NOT NULL,
    `name` TEXT,
    `thumbnail` BLOB,
    `thumbnail_mime_type` TEXT,
    PRIMARY KEY (`offer_id`, `requested`),
    FOREIGN KEY (`offer_id`) REFERENCES `offers`(`offer_id`) ON DELETE CASCADE
);

CREATE INDEX `nft_offer_id` ON `offer_nfts` (`offer_id`);

CREATE TABLE `offer_cats` (
    `offer_id` BLOB NOT NULL,
    `requested` BOOLEAN NOT NULL,
    `asset_id` BLOB NOT NULL,
    `amount` BLOB NOT NULL,
    `royalty` BLOB NOT NULL,
    `name` TEXT,
    `ticker` TEXT,
    `icon` TEXT,
    PRIMARY KEY (`offer_id`, `requested`),
    FOREIGN KEY (`offer_id`) REFERENCES `offers`(`offer_id`) ON DELETE CASCADE
);

CREATE INDEX `cat_offer_id` ON `offer_cats` (`offer_id`);
