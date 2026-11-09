CREATE TABLE IF NOT EXISTS sage_app_storages (
    id INTEGER PRIMARY KEY,
    storage_json TEXT NOT NULL,
    created_at_ms INTEGER NOT NULL,
    updated_at_ms INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS sage_app_origins (
    id INTEGER PRIMARY KEY,
    origin_id TEXT NOT NULL,
    storage_id INTEGER NOT NULL,
    may_contain_secrets INTEGER NOT NULL DEFAULT 0,
    created_at_ms INTEGER NOT NULL,
    updated_at_ms INTEGER NOT NULL,

    FOREIGN KEY(storage_id) REFERENCES sage_app_storages(id)
);

CREATE TABLE IF NOT EXISTS sage_apps (
    app_id TEXT PRIMARY KEY,
    storage_id INTEGER NOT NULL,
    origin_row_id INTEGER NOT NULL,
    created_at_ms INTEGER NOT NULL,
    updated_at_ms INTEGER NOT NULL,

    FOREIGN KEY(storage_id) REFERENCES sage_app_storages(id),
    FOREIGN KEY(origin_row_id) REFERENCES sage_app_origins(id)
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_sage_app_origins_origin_id
    ON sage_app_origins(origin_id);

CREATE INDEX IF NOT EXISTS idx_sage_app_origins_storage_id
    ON sage_app_origins(storage_id);
