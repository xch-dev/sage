-- Drop existing triggers to recreate them
DROP TRIGGER IF EXISTS nfts_ai;
DROP TRIGGER IF EXISTS nfts_ad;
DROP TRIGGER IF EXISTS nfts_au;

-- Clear and rebuild FTS table
DELETE FROM nft_name_fts;

-- Repopulate FTS table with existing NFT names
INSERT INTO nft_name_fts(name, nft_rowid, launcher_id)
SELECT name, rowid, launcher_id
FROM nfts
WHERE name IS NOT NULL 
AND name != '';

-- Recreate triggers with duplicate prevention
CREATE TRIGGER nfts_ai AFTER INSERT ON nfts BEGIN
  -- Remove any existing entry for this launcher_id first
  DELETE FROM nft_name_fts 
  WHERE launcher_id = NEW.launcher_id;
  
  -- Then insert the new entry if it has a name
  INSERT INTO nft_name_fts(name, nft_rowid, launcher_id)
  SELECT NEW.name, NEW.rowid, NEW.launcher_id
  WHERE NEW.name IS NOT NULL 
  AND NEW.name != '';
END;

CREATE TRIGGER nfts_ad AFTER DELETE ON nfts BEGIN
  DELETE FROM nft_name_fts WHERE nft_rowid = OLD.rowid;
END;

CREATE TRIGGER nfts_au AFTER UPDATE ON nfts BEGIN
  -- Remove the old entry
  DELETE FROM nft_name_fts WHERE launcher_id = NEW.launcher_id;
  
  -- Insert the updated entry if it has a name
  INSERT INTO nft_name_fts(name, nft_rowid, launcher_id)
  SELECT NEW.name, NEW.rowid, NEW.launcher_id
  WHERE NEW.name IS NOT NULL 
  AND NEW.name != '';
END; 
