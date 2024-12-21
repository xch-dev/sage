CREATE TABLE `offer_nfts_2` (
    `offer_id` BLOB NOT NULL,
    `requested` BOOLEAN NOT NULL,
    `launcher_id` BLOB NOT NULL,
    `royalty_puzzle_hash` BLOB NOT NULL,
    `royalty_ten_thousandths` INTEGER NOT NULL,
    `name` TEXT,
    `thumbnail` BLOB,
    `thumbnail_mime_type` TEXT,
    PRIMARY KEY (`offer_id`, `launcher_id`, `requested`),
    FOREIGN KEY (`offer_id`) REFERENCES `offers`(`offer_id`) ON DELETE CASCADE
);

CREATE TABLE `offer_cats_2` (
    `offer_id` BLOB NOT NULL,
    `requested` BOOLEAN NOT NULL,
    `asset_id` BLOB NOT NULL,
    `amount` BLOB NOT NULL,
    `royalty` BLOB NOT NULL,
    `name` TEXT,
    `ticker` TEXT,
    `icon` TEXT,
    PRIMARY KEY (`offer_id`, `asset_id`, `requested`),
    FOREIGN KEY (`offer_id`) REFERENCES `offers`(`offer_id`) ON DELETE CASCADE
);

INSERT INTO `offer_nfts_2` SELECT * FROM `offer_nfts`;
INSERT INTO `offer_cats_2` SELECT * FROM `offer_cats`;

DROP TABLE `offer_nfts`;
DROP TABLE `offer_cats`;

ALTER TABLE `offer_nfts_2` RENAME TO `offer_nfts`;
ALTER TABLE `offer_cats_2` RENAME TO `offer_cats`;

CREATE INDEX `nft_offer_id` ON `offer_nfts` (`offer_id`);
CREATE INDEX `cat_offer_id` ON `offer_cats` (`offer_id`);
