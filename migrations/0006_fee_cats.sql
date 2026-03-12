ALTER TABLE assets ADD COLUMN fee_issuer_puzzle_hash BLOB;
ALTER TABLE assets ADD COLUMN fee_basis_points INTEGER;
ALTER TABLE assets ADD COLUMN fee_min_fee INTEGER;
ALTER TABLE assets ADD COLUMN fee_allow_zero_price BOOLEAN;
ALTER TABLE assets ADD COLUMN fee_allow_revoke_fee_bypass BOOLEAN;
