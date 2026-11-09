ALTER TABLE sage_apps ADD COLUMN app_dir TEXT;
ALTER TABLE sage_apps ADD COLUMN source_json TEXT;
ALTER TABLE sage_apps ADD COLUMN granted_permissions_json TEXT;
ALTER TABLE sage_apps ADD COLUMN active_snapshot_json TEXT;
ALTER TABLE sage_apps ADD COLUMN wallet_scope_json TEXT;
ALTER TABLE sage_apps ADD COLUMN pending_update_json TEXT;
