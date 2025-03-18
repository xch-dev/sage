CREATE TABLE `nft_thumbnails` (
    `hash` BLOB NOT NULL PRIMARY KEY,
    `icon` BLOB NOT NULL,
    `thumbnail` BLOB NOT NULL
);

DELETE FROM `nft_data`;
UPDATE `nft_uris` SET `checked` = 0;
