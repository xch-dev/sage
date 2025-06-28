INSERT INTO assets (id, kind, hash, name, description, icon_url, is_visible, is_pending, created_height) 
VALUES (0, 0, x'0000000000000000000000000000000000000000000000000000000000000000', 'Chia', 'The primary asset on the Chia blockchain.', 'https://icons.dexie.space/xch.webp', TRUE, FALSE, NULL);

INSERT INTO tokens (id, asset_id, ticker, precision)
VALUES (0, 0, 'XCH', 12);

INSERT INTO collections (id, name, hash, uuid, minter_hash, is_visible)
VALUES (0, 'No Collection', x'0000000000000000000000000000000000000000000000000000000000000000', '00000000-0000-0000-0000-000000000000', x'0000000000000000000000000000000000000000000000000000000000000000', TRUE);
