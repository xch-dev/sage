use std::collections::BTreeSet;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use specta::Type;
use sqlx::{Row, sqlite::SqliteRow};

use crate::{AppsDb, SageAgent, SageAppWalletScope, UserBridgeCapability, unix_timestamp_ms};

const AGENT_SERVER_ENABLED_KEY: &str = "agent_server_enabled";
const AGENT_SERVER_PORT_KEY: &str = "agent_server_port";
const DEFAULT_AGENT_SERVER_PORT: u16 = 9258;

const AGENT_COLUMNS: &str = "agent_id, name, capabilities_json, wallet_scope_json, \
     created_at_ms, updated_at_ms, last_used_at_ms";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
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

impl AppsDb {
    pub async fn insert_agent(&self, agent: &SageAgent, token_hash: &str) -> Result<()> {
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

    pub async fn list_agents(&self) -> Result<Vec<SageAgent>> {
        let rows = sqlx::query(&format!(
            "SELECT {AGENT_COLUMNS} FROM sage_agents ORDER BY name COLLATE NOCASE, agent_id"
        ))
        .fetch_all(&self.pool)
        .await
        .context("failed to list agents")?;

        rows.iter().map(agent_from_row).collect()
    }

    pub async fn get_agent(&self, id: &str) -> Result<Option<SageAgent>> {
        let row = sqlx::query(&format!(
            "SELECT {AGENT_COLUMNS} FROM sage_agents WHERE agent_id = ?"
        ))
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .context("failed to read agent")?;

        row.as_ref().map(agent_from_row).transpose()
    }

    pub async fn find_agent_by_token_hash(&self, token_hash: &str) -> Result<Option<SageAgent>> {
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
        capabilities: &BTreeSet<UserBridgeCapability>,
        wallet_scope: &SageAppWalletScope,
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

    pub async fn get_agent_server_settings(&self) -> Result<AgentServerSettings> {
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

    pub async fn set_agent_server_settings(
        &self,
        settings: AgentServerSettings,
    ) -> Result<AgentServerSettings> {
        self.write_setting(AGENT_SERVER_ENABLED_KEY, &settings.enabled)
            .await?;
        self.write_setting(AGENT_SERVER_PORT_KEY, &settings.port)
            .await?;

        Ok(settings)
    }
}

fn agent_from_row(row: &SqliteRow) -> Result<SageAgent> {
    Ok(SageAgent {
        id: row.try_get("agent_id")?,
        name: row.try_get("name")?,
        capabilities: serde_json::from_str(&row.try_get::<String, _>("capabilities_json")?)
            .context("failed to deserialize agent capabilities")?,
        wallet_scope: serde_json::from_str(&row.try_get::<String, _>("wallet_scope_json")?)
            .context("failed to deserialize agent wallet scope")?,
        created_at_ms: row.try_get("created_at_ms")?,
        updated_at_ms: row.try_get("updated_at_ms")?,
        last_used_at_ms: row.try_get("last_used_at_ms")?,
    })
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    fn sample(name: &str) -> SageAgent {
        SageAgent {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            capabilities: BTreeSet::from([UserBridgeCapability::ALL[0]]),
            wallet_scope: SageAppWalletScope::AllWallets,
            created_at_ms: 1,
            updated_at_ms: 1,
            last_used_at_ms: None,
        }
    }

    #[tokio::test]
    async fn insert_then_find_by_hash() {
        let base = tempdir().unwrap();
        let db = AppsDb::initialize(base.path()).await.unwrap();
        let agent = sample("a");
        db.insert_agent(&agent, "h1").await.unwrap();

        assert_eq!(
            db.find_agent_by_token_hash("h1").await.unwrap(),
            Some(agent.clone())
        );
        assert_eq!(db.get_agent(&agent.id).await.unwrap(), Some(agent));
    }

    #[tokio::test]
    async fn find_by_unknown_hash_is_none() {
        let base = tempdir().unwrap();
        let db = AppsDb::initialize(base.path()).await.unwrap();
        assert!(db.find_agent_by_token_hash("nope").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn update_changes_grants_and_scope() {
        let base = tempdir().unwrap();
        let db = AppsDb::initialize(base.path()).await.unwrap();
        let agent = sample("a");
        db.insert_agent(&agent, "h1").await.unwrap();

        let caps = BTreeSet::new();
        let scope = SageAppWalletScope::SelectedWallets {
            fingerprints: vec![7],
        };
        assert!(
            db.update_agent(&agent.id, "b", &caps, &scope)
                .await
                .unwrap()
        );
        assert!(
            !db.update_agent("missing", "b", &caps, &scope)
                .await
                .unwrap()
        );

        let got = db.get_agent(&agent.id).await.unwrap().unwrap();
        assert_eq!(got.name, "b");
        assert_eq!(got.capabilities, caps);
        assert_eq!(got.wallet_scope, scope);
        assert!(got.is_wallet_in_scope(7));
        assert!(!got.is_wallet_in_scope(8));
    }

    #[tokio::test]
    async fn rotate_invalidates_old_hash() {
        let base = tempdir().unwrap();
        let db = AppsDb::initialize(base.path()).await.unwrap();
        let agent = sample("a");
        db.insert_agent(&agent, "old").await.unwrap();

        assert!(db.set_agent_token_hash(&agent.id, "new").await.unwrap());
        assert!(db.find_agent_by_token_hash("old").await.unwrap().is_none());
        assert!(db.find_agent_by_token_hash("new").await.unwrap().is_some());
        assert!(!db.set_agent_token_hash("missing", "x").await.unwrap());
    }

    #[tokio::test]
    async fn delete_removes_agent() {
        let base = tempdir().unwrap();
        let db = AppsDb::initialize(base.path()).await.unwrap();
        let agent = sample("a");
        db.insert_agent(&agent, "h1").await.unwrap();

        assert!(db.delete_agent(&agent.id).await.unwrap());
        assert!(!db.delete_agent(&agent.id).await.unwrap());
        assert!(db.get_agent(&agent.id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn touch_sets_last_used() {
        let base = tempdir().unwrap();
        let db = AppsDb::initialize(base.path()).await.unwrap();
        let agent = sample("a");
        db.insert_agent(&agent, "h1").await.unwrap();

        db.touch_agent_last_used(&agent.id, 42).await.unwrap();
        let got = db.get_agent(&agent.id).await.unwrap().unwrap();
        assert_eq!(got.last_used_at_ms, Some(42));
    }

    #[tokio::test]
    async fn list_is_sorted_by_name() {
        let base = tempdir().unwrap();
        let db = AppsDb::initialize(base.path()).await.unwrap();
        db.insert_agent(&sample("zeta"), "h1").await.unwrap();
        db.insert_agent(&sample("alpha"), "h2").await.unwrap();

        let names: Vec<_> = db
            .list_agents()
            .await
            .unwrap()
            .into_iter()
            .map(|a| a.name)
            .collect();
        assert_eq!(names, ["alpha", "zeta"]);
    }

    #[tokio::test]
    async fn server_settings_default_off_9258() {
        let base = tempdir().unwrap();
        let db = AppsDb::initialize(base.path()).await.unwrap();
        assert_eq!(
            db.get_agent_server_settings().await.unwrap(),
            AgentServerSettings {
                enabled: false,
                port: 9258
            }
        );

        let next = AgentServerSettings {
            enabled: true,
            port: 9300,
        };
        assert_eq!(db.set_agent_server_settings(next).await.unwrap(), next);
        assert_eq!(db.get_agent_server_settings().await.unwrap(), next);
    }
}
