CREATE TABLE IF NOT EXISTS sage_app_settings (
     key TEXT PRIMARY KEY,
     value_json TEXT NOT NULL,
     updated_at_ms INTEGER NOT NULL
);
