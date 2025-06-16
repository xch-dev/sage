-- columns specified with UNIQUE also create an auto-index
CREATE INDEX idx_transactions_height ON transactions(height);
CREATE INDEX idx_transaction_coins_transaction_id ON transaction_coins(transaction_id);
CREATE INDEX idx_nft_data_nft_id ON nft_data(nft_id);
CREATE INDEX idx_nfts_collection_id ON nfts(collection_id);
CREATE INDEX idx_derivations_index ON derivations(`index`, hardened);
CREATE INDEX idx_derivations_key ON derivations(synthetic_key);
CREATE INDEX idx_coins_asset_id ON coins(asset_id);
CREATE INDEX idx_coins_created_height ON coins(created_height);
CREATE INDEX idx_coins_hash ON coins(hash);
CREATE INDEX idx_coins_puzzle_hash ON coins(puzzle_hash);
CREATE INDEX idx_coins_hint ON coins(hint);
CREATE INDEX idx_coins_amount ON coins(amount);

/*
    Asset search indices
*/
CREATE INDEX idx_asset_name ON assets (is_visible DESC, is_pending DESC, name ASC, hash ASC);
CREATE INDEX idx_nft_collection ON nfts (is_owned, collection_id, edition_number ASC);
CREATE INDEX idx_nft_minter_did ON nfts (is_owned, minter_did, edition_number ASC);
CREATE INDEX idx_nft_owner_did ON nfts (is_owned, owner_did, edition_number ASC); 
CREATE INDEX idx_token_ticker ON tokens (ticker ASC); 