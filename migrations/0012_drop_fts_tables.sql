-- Drop triggers first to prevent any issues during table deletion
DROP TRIGGER IF EXISTS nfts_ai;
DROP TRIGGER IF EXISTS nfts_ad;
DROP TRIGGER IF EXISTS nfts_au;

-- Drop the FTS virtual table and its shadow tables
DROP TABLE IF EXISTS nft_name_fts;
