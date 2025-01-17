DROP INDEX `nft_did_name`;
DROP INDEX `nft_did_recent`;

CREATE INDEX `nft_minter_did_name` ON `nfts` (
    `is_owned`,
    `minter_did`,
    `visible` DESC,
    `is_pending` DESC,
    `is_named` DESC,
    `name` ASC,
    `launcher_id` ASC
);

CREATE INDEX `nft_minter_did_recent` ON `nfts` (
    `is_owned`, 
    `minter_did`,
    `visible` DESC,
    `is_pending` DESC,
    `created_height` DESC,
    `launcher_id` ASC
);

CREATE INDEX `nft_owner_did_name` ON `nfts` (
    `is_owned`,
    `owner_did`,
    `visible` DESC,
    `is_pending` DESC,
    `is_named` DESC,
    `name` ASC,
    `launcher_id` ASC
);

CREATE INDEX `nft_owner_did_recent` ON `nfts` (
    `is_owned`, 
    `owner_did`,
    `visible` DESC,
    `is_pending` DESC,
    `created_height` DESC,
    `launcher_id` ASC
);
