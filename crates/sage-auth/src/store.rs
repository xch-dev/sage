use std::{
    collections::BTreeSet,
    fs,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use sqlx::{
    Row, SqlitePool,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteRow},
};

use crate::{Agent, Capability, WalletScope};

const DB_FILE: &str = "agents.sqlite";
const AGENT_SERVER_ENABLED_KEY: &str = "agent_server_enabled";
const AGENT_SERVER_PORT_KEY: &str = "agent_server_port";

/// The port the agent server listens on unless the user changes it.
pub const DEFAULT_AGENT_SERVER_PORT: u16 = 9258;

const AGENT_COLUMNS: &str = "agent_id, name, capabilities_json, wallet_scope_json, \
     created_at_ms, updated_at_ms, last_used_at_ms";

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("src/migrations");

/// Whether the agent server runs, and where.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct AgentServerSettings {
    pub enabled: bool,
    pub port: u16,
}

impl Default for AgentServerSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            port: DEFAULT_AGENT_SERVER_PORT,
        }
    }
}

/// The current time in milliseconds since the Unix epoch.
pub fn unix_timestamp_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time is before UNIX_EPOCH")
        .as_millis() as i64
}

/// Storage in a `SQLite` file for agents and the agent server settings.
#[derive(Debug, Clone)]
pub struct AgentStore {
    pool: SqlitePool,
}

impl AgentStore {
    pub async fn open(base_path: &Path) -> Result<Self> {
        fs::create_dir_all(base_path).with_context(|| {
            format!(
                "failed to create agent store directory {}",
                base_path.display()
            )
        })?;

        let db_path = base_path.join(DB_FILE);

        let options = SqliteConnectOptions::new()
            .filename(&db_path)
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .foreign_keys(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(8)
            .connect_with(options)
            .await
            .with_context(|| format!("failed to open agent database {}", db_path.display()))?;

        MIGRATOR.run(&pool).await.with_context(|| {
            format!(
                "failed to run agent database migrations {}",
                db_path.display()
            )
        })?;

        Ok(Self { pool })
    }

    #[cfg(test)]
    pub(crate) fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Closes the database; later queries fail.
    pub async fn close(&self) {
        self.pool.close().await;
    }

    pub async fn insert_agent(&self, agent: &Agent, token_hash: &str) -> Result<()> {
        sqlx::query(
            r"
            INSERT INTO sage_agents (
                agent_id,
                name,
                token_hash,
                capabilities_json,
                wallet_scope_json,
                created_at_ms,
                updated_at_ms,
                last_used_at_ms
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            ",
        )
        .bind(&agent.id)
        .bind(&agent.name)
        .bind(token_hash)
        .bind(serde_json::to_string(&agent.capabilities)?)
        .bind(serde_json::to_string(&agent.wallet_scope)?)
        .bind(agent.created_at_ms)
        .bind(agent.updated_at_ms)
        .bind(agent.last_used_at_ms)
        .execute(&self.pool)
        .await
        .context("failed to insert agent")?;

        Ok(())
    }

    pub async fn list_agents(&self) -> Result<Vec<Agent>> {
        let rows = sqlx::query(&format!(
            "SELECT {AGENT_COLUMNS} FROM sage_agents ORDER BY name COLLATE NOCASE, agent_id"
        ))
        .fetch_all(&self.pool)
        .await
        .context("failed to list agents")?;

        rows.iter().map(agent_from_row).collect()
    }

    pub async fn get_agent(&self, id: &str) -> Result<Option<Agent>> {
        let row = sqlx::query(&format!(
            "SELECT {AGENT_COLUMNS} FROM sage_agents WHERE agent_id = ?"
        ))
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .context("failed to read agent")?;

        row.as_ref().map(agent_from_row).transpose()
    }

    pub async fn find_agent_by_token_hash(&self, token_hash: &str) -> Result<Option<Agent>> {
        let row = sqlx::query(&format!(
            "SELECT {AGENT_COLUMNS} FROM sage_agents WHERE token_hash = ?"
        ))
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await
        .context("failed to look up agent by token")?;

        row.as_ref().map(agent_from_row).transpose()
    }

    pub async fn update_agent(
        &self,
        id: &str,
        name: &str,
        capabilities: &BTreeSet<Capability>,
        wallet_scope: &WalletScope,
    ) -> Result<bool> {
        let result = sqlx::query(
            r"
            UPDATE sage_agents
            SET name = ?,
                capabilities_json = ?,
                wallet_scope_json = ?,
                updated_at_ms = ?
            WHERE agent_id = ?
            ",
        )
        .bind(name)
        .bind(serde_json::to_string(capabilities)?)
        .bind(serde_json::to_string(wallet_scope)?)
        .bind(unix_timestamp_ms())
        .bind(id)
        .execute(&self.pool)
        .await
        .context("failed to update agent")?;

        Ok(result.rows_affected() > 0)
    }

    /// Drops wallets that no longer exist from every agent's scope, and returns the agents.
    pub async fn retain_agent_wallets(&self, existing: &[u32]) -> Result<Vec<Agent>> {
        let mut agents = self.list_agents().await?;
        let mut edited_meanwhile = false;

        for agent in &mut agents {
            let WalletScope::SelectedWallets { fingerprints } = &agent.wallet_scope else {
                continue;
            };

            let kept: Vec<u32> = fingerprints
                .iter()
                .copied()
                .filter(|fingerprint| existing.contains(fingerprint))
                .collect();

            if kept.len() != fingerprints.len() {
                let scope = WalletScope::SelectedWallets { fingerprints: kept };
                let now = unix_timestamp_ms();
                if self
                    .replace_agent_wallet_scope(&agent.id, &agent.wallet_scope, &scope, now)
                    .await?
                {
                    agent.wallet_scope = scope;
                    agent.updated_at_ms = now;
                } else {
                    edited_meanwhile = true;
                }
            }
        }

        if edited_meanwhile {
            return self.list_agents().await;
        }

        Ok(agents)
    }

    /// Changes only the wallet scope, and only if it is still `expected`, so that an edit saved
    /// since the agent was read isn't overwritten.
    async fn replace_agent_wallet_scope(
        &self,
        id: &str,
        expected: &WalletScope,
        wallet_scope: &WalletScope,
        updated_at_ms: i64,
    ) -> Result<bool> {
        let result = sqlx::query(
            r"
            UPDATE sage_agents
            SET wallet_scope_json = ?,
                updated_at_ms = ?
            WHERE agent_id = ? AND wallet_scope_json = ?
            ",
        )
        .bind(serde_json::to_string(wallet_scope)?)
        .bind(updated_at_ms)
        .bind(id)
        .bind(serde_json::to_string(expected)?)
        .execute(&self.pool)
        .await
        .context("failed to update agent wallet scope")?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn set_agent_token_hash(&self, id: &str, token_hash: &str) -> Result<bool> {
        let result = sqlx::query(
            r"
            UPDATE sage_agents
            SET token_hash = ?,
                updated_at_ms = ?
            WHERE agent_id = ?
            ",
        )
        .bind(token_hash)
        .bind(unix_timestamp_ms())
        .bind(id)
        .execute(&self.pool)
        .await
        .context("failed to rotate agent token")?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn delete_agent(&self, id: &str) -> Result<bool> {
        let result = sqlx::query("DELETE FROM sage_agents WHERE agent_id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .context("failed to delete agent")?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn touch_agent_last_used(&self, id: &str, at_ms: i64) -> Result<()> {
        sqlx::query("UPDATE sage_agents SET last_used_at_ms = ? WHERE agent_id = ?")
            .bind(at_ms)
            .bind(id)
            .execute(&self.pool)
            .await
            .context("failed to update agent last used time")?;

        Ok(())
    }

    pub async fn get_server_settings(&self) -> Result<AgentServerSettings> {
        let defaults = AgentServerSettings::default();

        Ok(AgentServerSettings {
            enabled: self
                .read_setting(AGENT_SERVER_ENABLED_KEY)
                .await?
                .unwrap_or(defaults.enabled),
            port: self
                .read_setting(AGENT_SERVER_PORT_KEY)
                .await?
                .unwrap_or(defaults.port),
        })
    }

    pub async fn set_server_settings(
        &self,
        settings: AgentServerSettings,
    ) -> Result<AgentServerSettings> {
        self.write_setting(AGENT_SERVER_ENABLED_KEY, &settings.enabled)
            .await?;
        self.write_setting(AGENT_SERVER_PORT_KEY, &settings.port)
            .await?;

        Ok(settings)
    }

    async fn read_setting<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>> {
        let row = sqlx::query("SELECT value FROM settings WHERE key = ?")
            .bind(key)
            .fetch_optional(&self.pool)
            .await
            .with_context(|| format!("failed to read setting {key}"))?;

        row.map(|row| {
            serde_json::from_str(&row.try_get::<String, _>("value")?)
                .with_context(|| format!("failed to deserialize setting {key}"))
        })
        .transpose()
    }

    async fn write_setting<T: Serialize>(&self, key: &str, value: &T) -> Result<()> {
        sqlx::query(
            r"
            INSERT INTO settings (key, value)
            VALUES (?, ?)
            ON CONFLICT(key) DO UPDATE SET value = excluded.value
            ",
        )
        .bind(key)
        .bind(serde_json::to_string(value)?)
        .execute(&self.pool)
        .await
        .with_context(|| format!("failed to persist setting {key}"))?;

        Ok(())
    }
}

fn agent_from_row(row: &SqliteRow) -> Result<Agent> {
    // Unknown capability keys (e.g. from a newer version) are dropped, not fatal.
    let keys: Vec<String> = serde_json::from_str(&row.try_get::<String, _>("capabilities_json")?)
        .context("failed to deserialize agent capabilities")?;
    let capabilities: BTreeSet<Capability> = keys
        .iter()
        .filter_map(|key| Capability::from_key(key))
        .collect();

    Ok(Agent {
        id: row.try_get("agent_id")?,
        name: row.try_get("name")?,
        capabilities,
        wallet_scope: serde_json::from_str(&row.try_get::<String, _>("wallet_scope_json")?)
            .context("failed to deserialize agent wallet scope")?,
        created_at_ms: row.try_get("created_at_ms")?,
        updated_at_ms: row.try_get("updated_at_ms")?,
        last_used_at_ms: row.try_get("last_used_at_ms")?,
    })
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    async fn store() -> (AgentStore, TempDir) {
        let dir = TempDir::new().unwrap();
        (AgentStore::open(dir.path()).await.unwrap(), dir)
    }

    fn agent(id: &str, caps: &[Capability]) -> Agent {
        Agent {
            id: id.to_string(),
            name: id.to_string(),
            capabilities: caps.iter().copied().collect(),
            wallet_scope: WalletScope::AllWallets,
            created_at_ms: 1,
            updated_at_ms: 1,
            last_used_at_ms: None,
        }
    }

    #[tokio::test]
    async fn queries_fail_after_close() {
        let (store, _dir) = store().await;
        store.close().await;
        assert!(store.list_agents().await.is_err());
    }

    fn sample(name: &str) -> Agent {
        agent(name, &[Capability::WalletGetCoins])
    }

    fn scoped(id: &str, fingerprints: &[u32]) -> Agent {
        Agent {
            wallet_scope: WalletScope::SelectedWallets {
                fingerprints: fingerprints.to_vec(),
            },
            ..sample(id)
        }
    }

    #[tokio::test]
    async fn insert_then_find_by_hash() {
        let (store, _dir) = store().await;
        let agent = sample("a");
        store.insert_agent(&agent, "h1").await.unwrap();

        assert_eq!(
            store.find_agent_by_token_hash("h1").await.unwrap(),
            Some(agent.clone())
        );
        assert_eq!(store.get_agent(&agent.id).await.unwrap(), Some(agent));
    }

    #[tokio::test]
    async fn deleted_wallets_are_dropped_from_scopes() {
        let (store, _dir) = store().await;
        let all = sample("all");
        let selected = scoped("selected", &[7, 8]);
        store.insert_agent(&all, "h1").await.unwrap();
        store.insert_agent(&selected, "h2").await.unwrap();

        let returned = store.retain_agent_wallets(&[8, 9]).await.unwrap();
        assert_eq!(returned, store.list_agents().await.unwrap());

        let scope = |id: &str| {
            let store = store.clone();
            let id = id.to_string();
            async move { store.get_agent(&id).await.unwrap().unwrap().wallet_scope }
        };
        assert_eq!(scope(&all.id).await, WalletScope::AllWallets);
        assert_eq!(
            scope(&selected.id).await,
            WalletScope::SelectedWallets {
                fingerprints: vec![8]
            }
        );
    }

    #[tokio::test]
    async fn scope_replacement_keeps_a_newer_edit() {
        let (store, _dir) = store().await;
        let stale = WalletScope::SelectedWallets {
            fingerprints: vec![7, 8],
        };
        let agent = scoped("a", &[7, 8]);
        store.insert_agent(&agent, "h1").await.unwrap();

        // The user removes every capability after the prune read the agent, but before it wrote.
        let caps = BTreeSet::new();
        store
            .update_agent(&agent.id, "a", &caps, &stale)
            .await
            .unwrap();
        let pruned = WalletScope::SelectedWallets {
            fingerprints: vec![8],
        };
        assert!(
            store
                .replace_agent_wallet_scope(&agent.id, &stale, &pruned, 2)
                .await
                .unwrap()
        );
        let got = store.get_agent(&agent.id).await.unwrap().unwrap();
        assert_eq!(got.capabilities, caps);
        assert_eq!(got.wallet_scope, pruned);

        // A scope the user changed since it was read is left alone.
        assert!(
            !store
                .replace_agent_wallet_scope(&agent.id, &stale, &WalletScope::AllWallets, 3)
                .await
                .unwrap()
        );
        let got = store.get_agent(&agent.id).await.unwrap().unwrap();
        assert_eq!(got.wallet_scope, pruned);
    }

    #[tokio::test]
    async fn find_by_unknown_hash_is_none() {
        let (store, _dir) = store().await;
        assert!(
            store
                .find_agent_by_token_hash("nope")
                .await
                .unwrap()
                .is_none()
        );
    }

    #[tokio::test]
    async fn update_changes_grants_and_scope() {
        let (store, _dir) = store().await;
        let agent = sample("a");
        store.insert_agent(&agent, "h1").await.unwrap();

        let caps = BTreeSet::new();
        let scope = WalletScope::SelectedWallets {
            fingerprints: vec![7],
        };
        assert!(
            store
                .update_agent(&agent.id, "b", &caps, &scope)
                .await
                .unwrap()
        );
        assert!(
            !store
                .update_agent("missing", "b", &caps, &scope)
                .await
                .unwrap()
        );

        let got = store.get_agent(&agent.id).await.unwrap().unwrap();
        assert_eq!(got.name, "b");
        assert_eq!(got.capabilities, caps);
        assert_eq!(got.wallet_scope, scope);
        assert!(got.wallet_scope.allows(7));
        assert!(!got.wallet_scope.allows(8));
    }

    #[tokio::test]
    async fn rotate_invalidates_old_hash() {
        let (store, _dir) = store().await;
        let agent = sample("a");
        store.insert_agent(&agent, "old").await.unwrap();

        assert!(store.set_agent_token_hash(&agent.id, "new").await.unwrap());
        assert!(
            store
                .find_agent_by_token_hash("old")
                .await
                .unwrap()
                .is_none()
        );
        assert!(
            store
                .find_agent_by_token_hash("new")
                .await
                .unwrap()
                .is_some()
        );
        assert!(!store.set_agent_token_hash("missing", "x").await.unwrap());
    }

    #[tokio::test]
    async fn delete_removes_agent() {
        let (store, _dir) = store().await;
        let agent = sample("a");
        store.insert_agent(&agent, "h1").await.unwrap();

        assert!(store.delete_agent(&agent.id).await.unwrap());
        assert!(!store.delete_agent(&agent.id).await.unwrap());
        assert!(store.get_agent(&agent.id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn touch_sets_last_used() {
        let (store, _dir) = store().await;
        let agent = sample("a");
        store.insert_agent(&agent, "h1").await.unwrap();

        store.touch_agent_last_used(&agent.id, 42).await.unwrap();
        let got = store.get_agent(&agent.id).await.unwrap().unwrap();
        assert_eq!(got.last_used_at_ms, Some(42));
    }

    #[tokio::test]
    async fn list_is_sorted_by_name() {
        let (store, _dir) = store().await;
        store.insert_agent(&sample("zeta"), "h1").await.unwrap();
        store.insert_agent(&sample("alpha"), "h2").await.unwrap();

        let names: Vec<_> = store
            .list_agents()
            .await
            .unwrap()
            .into_iter()
            .map(|a| a.name)
            .collect();
        assert_eq!(names, ["alpha", "zeta"]);
    }

    #[tokio::test]
    async fn unknown_stored_capability_is_dropped_not_fatal() {
        let (store, _dir) = store().await;
        store
            .insert_agent(&agent("a", &[Capability::WalletGetCoins]), "h")
            .await
            .unwrap();
        sqlx::query(
            "UPDATE sage_agents SET capabilities_json = '[\"wallet.get_coins\",\"wallet.send_xch\"]'",
        )
        .execute(store.pool())
        .await
        .unwrap();
        let agents = store.list_agents().await.unwrap();
        assert_eq!(
            agents[0].capabilities,
            BTreeSet::from([Capability::WalletGetCoins])
        );
    }

    #[tokio::test]
    async fn open_fails_on_a_corrupt_file() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("agents.sqlite"), b"not a database").unwrap();
        assert!(AgentStore::open(dir.path()).await.is_err());
    }

    #[tokio::test]
    async fn pruned_wallet_does_not_return_when_reimported() {
        let (store, _dir) = store().await;
        store
            .insert_agent(&scoped("a", &[7, 8]), "h")
            .await
            .unwrap();
        store.retain_agent_wallets(&[8]).await.unwrap();
        store.retain_agent_wallets(&[7, 8]).await.unwrap();
        let scope = store.get_agent("a").await.unwrap().unwrap().wallet_scope;
        assert_eq!(
            scope,
            WalletScope::SelectedWallets {
                fingerprints: vec![8]
            }
        );
    }

    #[tokio::test]
    async fn settings_default_then_round_trip() {
        let (store, _dir) = store().await;
        assert_eq!(
            store.get_server_settings().await.unwrap(),
            AgentServerSettings::default()
        );
        let next = AgentServerSettings {
            enabled: true,
            port: 9300,
        };
        store.set_server_settings(next).await.unwrap();
        assert_eq!(store.get_server_settings().await.unwrap(), next);
    }
}
