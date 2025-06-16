CREATE TRIGGER check_token_type 
BEFORE INSERT ON tokens 
BEGIN  
    SELECT CASE 
        WHEN (SELECT kind FROM assets WHERE id = NEW.asset_id) != 0 
        THEN RAISE(ABORT, 'Asset type must be token')  
    END; 
END;

CREATE TRIGGER check_nft_type 
BEFORE INSERT ON nfts 
BEGIN  
    SELECT CASE 
        WHEN (SELECT kind FROM assets WHERE id = NEW.asset_id) != 1  
        THEN RAISE(ABORT, 'Asset type must be nft')  
    END; 
END;

CREATE TRIGGER check_did_type 
BEFORE INSERT ON dids 
BEGIN  
    SELECT CASE 
        WHEN (SELECT kind FROM assets WHERE id = NEW.asset_id) != 2 
        THEN RAISE(ABORT, 'Asset type must be did')  
    END; 
END;