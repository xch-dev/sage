CREATE TRIGGER trigger_prevent_delete_xch_asset
    BEFORE DELETE ON assets
    FOR EACH ROW
    WHEN OLD.id = 0
BEGIN
    SELECT RAISE(ABORT, 'Cannot delete XCH asset');
END;

CREATE TRIGGER trigger_prevent_delete_default_collection
    BEFORE DELETE ON collections
    FOR EACH ROW
    WHEN OLD.id = 0
BEGIN
    SELECT RAISE(ABORT, 'Cannot delete default collection');
END;
