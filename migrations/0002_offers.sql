CREATE TABLE `offers` (
    `offer_id` BLOB NOT NULL PRIMARY KEY,
    `encoded_offer` TEXT NOT NULL,
    `expiration_height` INTEGER,
    `expiration_timestamp` BLOB,
    `status` INTEGER NOT NULL,
    `inserted_timestamp` BLOB NOT NULL
);

CREATE INDEX `offer_status` ON `offers` (`status`);
CREATE INDEX `offer_timestamp` ON `offers` (`inserted_timestamp` DESC);

CREATE TABLE `offer_coins` (
    `offer_id` BLOB NOT NULL,
    `coin_id` BLOB NOT NULL,
    PRIMARY KEY (`offer_id`, `coin_id`),
    FOREIGN KEY (`offer_id`) REFERENCES `offers`(`offer_id`) ON DELETE CASCADE
);

CREATE INDEX `offer_coin_id` ON `offer_coins` (`coin_id`);
