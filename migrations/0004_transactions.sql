ALTER TABLE `coin_states` ADD COLUMN `kind` INTEGER NOT NULL DEFAULT 0;
CREATE INDEX `coin_kind` ON `coin_states` (`kind`);
CREATE INDEX `coin_kind_spent` ON `coin_states` (`kind`, `spent_height` ASC);

UPDATE `coin_states` SET `kind` = 1 WHERE `coin_id` IN (SELECT `coin_id` FROM `p2_coins`);
DROP TABLE `p2_coins`;

UPDATE `coin_states` SET `kind` = 2 WHERE `coin_id` IN (SELECT `coin_id` FROM `cat_coins`);
UPDATE `coin_states` SET `kind` = 3 WHERE `coin_id` IN (SELECT `coin_id` FROM `nft_coins`);
UPDATE `coin_states` SET `kind` = 4 WHERE `coin_id` IN (SELECT `coin_id` FROM `did_coins`);
