CREATE VIEW wallet_nfts AS
  SELECT        
      wallet_coins.*, nfts.minter_hash, owner_hash, metadata, metadata_updater_puzzle_hash,
      royalty_puzzle_hash, royalty_basis_points, data_hash, metadata_hash, license_hash,
      edition_number, edition_total, nfts.collection_id
  FROM wallet_coins
  INNER JOIN nfts ON nfts.asset_id = wallet_coins.asset_id;