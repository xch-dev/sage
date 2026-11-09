ALTER TABLE sage_apps ADD COLUMN active_snapshot_manifest_hash TEXT;
ALTER TABLE sage_apps ADD COLUMN active_snapshot_dir TEXT;

ALTER TABLE sage_apps ADD COLUMN pending_update_app_url TEXT;
ALTER TABLE sage_apps ADD COLUMN pending_update_manifest_hash TEXT;
ALTER TABLE sage_apps ADD COLUMN pending_update_manifest_json TEXT;

ALTER TABLE sage_apps DROP COLUMN active_snapshot_json;
ALTER TABLE sage_apps DROP COLUMN pending_update_json;
