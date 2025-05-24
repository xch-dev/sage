-- Update existing name-based indexes to include edition_number for better sorting

-- Drop existing name-based indexes
DROP INDEX IF EXISTS `nft_name`;
DROP INDEX IF EXISTS `nft_col_name`;
DROP INDEX IF EXISTS `nft_minter_did_name`;
DROP INDEX IF EXISTS `nft_owner_did_name`;

-- Recreate them with edition_number included
CREATE INDEX `nft_name` ON `nfts` (`is_owned`, `visible` DESC, `is_pending` DESC, `is_named` DESC, `name` ASC, `edition_number` ASC, `launcher_id` ASC);
CREATE INDEX `nft_col_name` ON `nfts` (`is_owned`, `collection_id`, `visible` DESC, `is_pending` DESC, `is_named` DESC, `name` ASC, `edition_number` ASC, `launcher_id` ASC);
CREATE INDEX `nft_minter_did_name` ON `nfts` (
    `is_owned`,
    `minter_did`,
    `visible` DESC,
    `is_pending` DESC,
    `is_named` DESC,
    `name` ASC,
    `edition_number` ASC,
    `launcher_id` ASC
);
CREATE INDEX `nft_owner_did_name` ON `nfts` (
    `is_owned`,
    `owner_did`,
    `visible` DESC,
    `is_pending` DESC,
    `is_named` DESC,
    `name` ASC,
    `edition_number` ASC,
    `launcher_id` ASC
); 