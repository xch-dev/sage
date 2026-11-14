CREATE TABLE IF NOT EXISTS sage_agents (
    agent_id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    token_hash TEXT NOT NULL UNIQUE,
    capabilities_json TEXT NOT NULL,
    wallet_scope_json TEXT NOT NULL,
    created_at_ms INTEGER NOT NULL,
    updated_at_ms INTEGER NOT NULL,
    last_used_at_ms INTEGER
);
