/* foreign key indices */

-- this index plays a dual role as an FK index and helps with the puzzle queue
CREATE INDEX idx_coins_asset_spent_children ON coins (asset_id, spent_height, is_children_synced);
CREATE INDEX idx_coins_p2_puzzle_id ON coins(p2_puzzle_id);
CREATE INDEX idx_coins_created_height ON coins(created_height);
CREATE INDEX idx_coins_spent_height ON coins(spent_height);
CREATE INDEX idx_file_uris_file_id ON file_uris(file_id);
CREATE INDEX idx_mempool_coins_coin_id ON mempool_coins(coin_id);
CREATE INDEX idx_mempool_coins_mempool_item_id ON mempool_coins(mempool_item_id);
CREATE INDEX idx_mempool_spends_mempool_item_id ON mempool_spends(mempool_item_id);
CREATE INDEX idx_nfts_data_hash ON nfts(data_hash);
CREATE INDEX idx_nfts_metadata_hash ON nfts(metadata_hash);
CREATE INDEX idx_nfts_license_hash ON nfts(license_hash);
CREATE INDEX idx_nfts_collection_id ON nfts(collection_id);
CREATE INDEX idx_offer_assets_offer_id ON offer_assets(offer_id);
CREATE INDEX idx_offer_assets_asset_id ON offer_assets(asset_id);
CREATE INDEX idx_offer_coins_offer_id ON offer_coins(offer_id);
CREATE INDEX idx_offer_coins_coin_id ON offer_coins(coin_id);
CREATE INDEX idx_options_underlying_asset_id ON options(underlying_asset_id);
CREATE INDEX idx_options_strike_asset_id ON options(strike_asset_id);
CREATE INDEX idx_p2_options_option_asset_id ON p2_options(option_asset_id);
CREATE INDEX idx_resized_images_file_id ON resized_images(file_id);
CREATE INDEX idx_peaks ON blocks(is_peak DESC, height DESC);

/* search and ordering indices */
CREATE INDEX idx_assets_name ON assets(name ASC);
CREATE INDEX idx_assets_ticker ON assets(ticker);
CREATE INDEX idx_clawbacks_sender_puzzle_hash ON clawbacks(sender_puzzle_hash);
CREATE INDEX idx_clawbacks_receiver_puzzle_hash ON clawbacks(receiver_puzzle_hash);
CREATE INDEX idx_clawbacks_expiration_seconds ON clawbacks(expiration_seconds);
CREATE INDEX idx_nfts_minter_hash ON nfts(minter_hash);
CREATE INDEX idx_nfts_owner_hash ON nfts(owner_hash);
CREATE INDEX idx_nfts_edition_number ON nfts(edition_number ASC);
CREATE INDEX idx_options_underlying_coin_hash ON options(underlying_coin_hash);
CREATE INDEX idx_public_keys_key ON public_keys(key);
CREATE INDEX idx_public_keys_derivation_index_hardened ON public_keys(derivation_index, is_hardened);

