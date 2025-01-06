-- Add index for DID-based name searches
CREATE INDEX `nft_did_name` ON `nfts` (
    `is_owned`,
    `minter_did`,
    `visible` DESC,
    `is_pending` DESC,
    `is_named` DESC,
    `name` ASC,
    `launcher_id` ASC
);

-- Add index for DID-based recent searches
CREATE INDEX `nft_did_recent` ON `nfts` (
    `is_owned`, 
    `minter_did`,
    `visible` DESC,
    `is_pending` DESC,
    `created_height` DESC,
    `launcher_id` ASC
);

-- Create FTS5 virtual table for name searching
CREATE VIRTUAL TABLE `nft_name_fts` USING fts5(
    name,
    nft_rowid UNINDEXED,  -- Store the nfts table rowid explicitly
    launcher_id UNINDEXED
);

-- Populate FTS table with existing NFT names
INSERT INTO nft_name_fts(name, nft_rowid, launcher_id)
SELECT name, rowid, launcher_id
FROM nfts
WHERE name IS NOT NULL AND name != '';

-- Modified triggers with additional safety checks
CREATE TRIGGER nfts_ai AFTER INSERT ON nfts BEGIN
  INSERT INTO nft_name_fts(name, nft_rowid, launcher_id)
  SELECT NEW.name, NEW.rowid, NEW.launcher_id
  WHERE NEW.name IS NOT NULL AND NEW.name != '';
END;

CREATE TRIGGER nfts_ad AFTER DELETE ON nfts BEGIN
  DELETE FROM nft_name_fts WHERE nft_rowid = OLD.rowid;
END;

CREATE TRIGGER nfts_au AFTER UPDATE ON nfts BEGIN
  DELETE FROM nft_name_fts WHERE nft_rowid = OLD.rowid;
  INSERT INTO nft_name_fts(name, nft_rowid, launcher_id)
  SELECT NEW.name, NEW.rowid, NEW.launcher_id
  WHERE NEW.name IS NOT NULL AND NEW.name != '';
END; 
