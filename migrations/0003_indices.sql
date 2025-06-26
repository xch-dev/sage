/* foreign key indices */
CREATE INDEX idx_nfts_data_hash ON nfts(data_hash);
CREATE INDEX idx_nfts_metadata_hash ON nfts(metadata_hash);
CREATE INDEX idx_nfts_license_hash ON nfts(license_hash);
CREATE INDEX idx_nfts_collection_id ON nfts(collection_id);
CREATE INDEX idx_file_uris_file_id ON file_uris(file_id);
CREATE INDEX idx_public_keys_p2_puzzle_id ON public_keys(p2_puzzle_id);
CREATE INDEX idx_clawbacks_p2_puzzle_id ON clawbacks(p2_puzzle_id);
CREATE INDEX idx_p2_options_p2_puzzle_id ON p2_options(p2_puzzle_id);
CREATE INDEX idx_coins_asset_id ON coins(asset_id);
CREATE INDEX idx_coins_p2_puzzle_id ON coins(p2_puzzle_id);
CREATE INDEX idx_coins_created_height ON coins(created_height);
CREATE INDEX idx_coins_spent_height ON coins(spent_height);
CREATE INDEX idx_offer_assets_offer_id ON offer_assets(offer_id);
CREATE INDEX idx_offer_assets_asset_id ON offer_assets(asset_id);
CREATE INDEX idx_offer_coins_offer_id ON offer_coins(offer_id);
CREATE INDEX idx_offer_coins_coin_id ON offer_coins(coin_id);
CREATE INDEX idx_transaction_coins_coin_id ON transaction_coins(coin_id);
CREATE INDEX idx_transaction_coins_transaction_id ON transaction_coins(transaction_id);
CREATE INDEX idx_options_underlying_asset_id ON options(underlying_asset_id);
CREATE INDEX idx_options_strike_asset_id ON options(strike_asset_id);

/* search indices */
CREATE INDEX idx_tokens_ticker ON tokens(ticker);
CREATE INDEX idx_assets_search_order ON assets(is_visible DESC, is_pending DESC, name ASC, created_height DESC);
CREATE INDEX idx_assets_name ON assets(name);
