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
    launcher_id UNINDEXED,  -- We store but don't index these columns
    content='nfts',
    content_rowid='rowid'
);

-- Populate FTS table with existing NFT names
INSERT INTO nft_name_fts(rowid, name, launcher_id)
SELECT rowid, name, launcher_id
FROM nfts
WHERE name IS NOT NULL;

-- Create triggers to keep FTS table in sync
CREATE TRIGGER nfts_ai AFTER INSERT ON nfts BEGIN
  INSERT INTO nft_name_fts(rowid, name, launcher_id)
  SELECT NEW.rowid, NEW.name, NEW.launcher_id
  WHERE NEW.name IS NOT NULL;
END;

CREATE TRIGGER nfts_ad AFTER DELETE ON nfts BEGIN
  INSERT INTO nft_name_fts(nft_name_fts, rowid, name, launcher_id)
  VALUES('delete', OLD.rowid, OLD.name, OLD.launcher_id);
END;

CREATE TRIGGER nfts_au AFTER UPDATE ON nfts BEGIN
  INSERT INTO nft_name_fts(nft_name_fts, rowid, name, launcher_id)
  VALUES('delete', OLD.rowid, OLD.name, OLD.launcher_id);
  INSERT INTO nft_name_fts(rowid, name, launcher_id)
  SELECT NEW.rowid, NEW.name, NEW.launcher_id
  WHERE NEW.name IS NOT NULL;
END; 
