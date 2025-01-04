ALTER TABLE `nft_uris` ADD COLUMN `hash_matches` BOOLEAN;

UPDATE `nft_uris` SET `hash_matches` = 1;

ALTER TABLE `nft_data` ADD COLUMN `hash_matches` BOOLEAN NOT NULL DEFAULT 1;
