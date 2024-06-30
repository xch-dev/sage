CREATE TABLE `derivations` (
    `p2_puzzle_hash` BLOB NOT NULL PRIMARY KEY,
    `index` INTEGER NOT NULL,
    `hardened` INTEGER NOT NULL CHECK (`hardened` IN (0, 1)),
    `synthetic_key` BLOB NOT NULL
);
