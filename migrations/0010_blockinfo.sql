CREATE TABLE IF NOT EXISTS `blockinfo` (
    `height` INTEGER NOT NULL PRIMARY KEY,
    `unix_time` BIGINT NOT NULL
);

CREATE INDEX `blockinfo_index` ON `blockinfo` (`height`);

ALTER TABLE `coin_states` ADD COLUMN `spent_unixtime` BIGINT;
ALTER TABLE `coin_states` ADD COLUMN `created_unixtime` BIGINT;
