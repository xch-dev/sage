INSERT INTO assets (id, kind, hash, name, ticker, precision, description, icon_url, is_visible, hidden_puzzle_hash) 
VALUES (0, 0, x'0000000000000000000000000000000000000000000000000000000000000000', 'Chia', 'XCH', 12, 'The primary asset on the Chia blockchain.', 'https://icons.dexie.space/xch.webp', TRUE, NULL);

INSERT INTO collections (id, name, hash, uuid, minter_hash, is_visible)
VALUES (0, 'No Collection', x'0000000000000000000000000000000000000000000000000000000000000000', '00000000-0000-0000-0000-000000000000', x'0000000000000000000000000000000000000000000000000000000000000000', TRUE);

INSERT INTO rust_migrations (version) VALUES (0);
