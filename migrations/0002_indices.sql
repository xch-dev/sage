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
