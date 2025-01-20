-- Drop existing triggers to recreate them
DROP TRIGGER IF EXISTS nfts_ai;
DROP TRIGGER IF EXISTS nfts_ad;
DROP TRIGGER IF EXISTS nfts_au;

-- Clear and rebuild FTS table
DELETE FROM nft_name_fts;

-- Repopulate FTS table with existing NFT names, avoiding duplicates
INSERT INTO nft_name_fts(name, nft_rowid, launcher_id)
SELECT DISTINCT name, rowid, launcher_id
FROM nfts 
WHERE name IS NOT NULL 
AND name != ''
AND visible = 1
AND is_pending = 0;

-- Recreate triggers with duplicate prevention
CREATE TRIGGER nfts_ai AFTER INSERT ON nfts BEGIN
  INSERT INTO nft_name_fts(name, nft_rowid, launcher_id)
  SELECT NEW.name, NEW.rowid, NEW.launcher_id
  WHERE NEW.name IS NOT NULL 
  AND NEW.name != ''
  AND NEW.visible = 1
  AND NEW.is_pending = 0
  AND NOT EXISTS (
    SELECT 1 FROM nft_name_fts 
    WHERE launcher_id = NEW.launcher_id
  );
END;

CREATE TRIGGER nfts_ad AFTER DELETE ON nfts BEGIN
  DELETE FROM nft_name_fts WHERE nft_rowid = OLD.rowid;
END;

CREATE TRIGGER nfts_au AFTER UPDATE ON nfts BEGIN
  DELETE FROM nft_name_fts WHERE nft_rowid = OLD.rowid;
  INSERT INTO nft_name_fts(name, nft_rowid, launcher_id)
  SELECT NEW.name, NEW.rowid, NEW.launcher_id
  WHERE NEW.name IS NOT NULL 
  AND NEW.name != ''
  AND NEW.visible = 1
  AND NEW.is_pending = 0
  AND NOT EXISTS (
    SELECT 1 FROM nft_name_fts 
    WHERE launcher_id = NEW.launcher_id
    AND nft_rowid != NEW.rowid
  );
END; 
