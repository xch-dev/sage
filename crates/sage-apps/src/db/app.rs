use std::path::Path;

use anyhow::{Context, Result};
use sqlx::{Row, SqliteConnection, sqlite::SqliteRow};

use crate::db::{AppsDb, AppsDbTx};
use crate::types::{CorruptedInstalledSageApp, ListedSageApp, SageAppCommon, SageAppIconView, SageAppIdentity, SageAppPackageManifest, SageAppSnapshot, SageAppStorage, SageAppUrl, SageAppWalletScope, SageGrantedPermissions, UserSageApp, UserSageAppPendingUpdate, UserSageAppSource};

impl AppsDb {
    pub async fn app_exists(&self, app_id: &str) -> Result<bool> {
        let row = sqlx::query(
            r#"
            SELECT 1
            FROM sage_apps
            WHERE app_id = ?
            LIMIT 1
            "#,
        )
            .bind(app_id)
            .fetch_optional(&self.pool)
            .await
            .with_context(|| format!("failed to check if app exists {app_id}"))?;

        Ok(row.is_some())
    }

    pub async fn get_user_app(&self, app_id: &str) -> Result<Option<UserSageApp>> {
        let mut conn = self
            .pool()
            .acquire()
            .await
            .context("failed to acquire apps db connection")?
            .detach();

        load_user_app_optional_from_conn(&mut conn, app_id).await
    }

    pub async fn list_installed_apps(&self) -> Result<Vec<ListedSageApp>> {
        let mut conn = self
            .pool()
            .acquire()
            .await
            .context("failed to acquire apps db connection")?
            .detach();

        let app_ids = list_user_app_ids_from_conn(&mut conn).await?;
        let mut listed = Vec::with_capacity(app_ids.len());

        for app_id in app_ids {
            match load_user_app_from_conn(&mut conn, &app_id).await {
                Ok(app) => listed.push(ListedSageApp::User(app)),
                Err(err) => {
                    listed.push(load_corrupted_user_app_from_conn(&mut conn, &app_id, err).await?);
                }
            }
        }

        Ok(listed)
    }
}

impl AppsDbTx {
    pub(crate) async fn load_user_app(&mut self, app_id: &str) -> Result<UserSageApp> {
        load_user_app_from_conn(&mut self.conn, app_id).await
    }

    pub(crate) async fn insert_user_app(
        &mut self,
        app: &UserSageApp,
        storage_id: i64,
        origin_row_id: i64,
    ) -> Result<()> {
        let common = app.common();
        let now = crate::utils::unix_timestamp_ms();

        sqlx::query(
            r#"
            INSERT INTO sage_apps (
                app_id,
                storage_id,
                origin_row_id,
                app_dir,
                source_json,
                granted_permissions_json,
                wallet_scope_json,
                active_snapshot_manifest_hash,
                active_snapshot_dir,
                pending_update_app_url,
                pending_update_manifest_hash,
                pending_update_manifest_json,
                created_at_ms,
                updated_at_ms
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
            .bind(common.id())
            .bind(storage_id)
            .bind(origin_row_id)
            .bind(common.app_dir())
            .bind(serde_json::to_string(app.source())?)
            .bind(serde_json::to_string(common.granted_permissions())?)
            .bind(serde_json::to_string(common.wallet_scope())?)
            .bind(common.active_snapshot().manifest_hash())
            .bind(common.active_snapshot().snapshot_dir())
            .bind(app.pending_update().map(|p| p.app_url().to_string()))
            .bind(app.pending_update().map(|p| p.manifest_hash().to_string()))
            .bind(app.pending_update().map(|p| serde_json::to_string(p.manifest())).transpose()?)
            .bind(now)
            .bind(now)
            .execute(&mut self.conn)
            .await
            .with_context(|| format!("failed to insert app {}", common.id()))?;

        Ok(())
    }

    pub(crate) async fn persist_user_app(&mut self, app: &UserSageApp) -> Result<()> {
        let common = app.common();
        let now = crate::utils::unix_timestamp_ms();

        sqlx::query(
            r#"
            UPDATE sage_app_origins
            SET may_contain_secrets = ?, updated_at_ms = ?
            WHERE id = (
                SELECT origin_row_id
                FROM sage_apps
                WHERE app_id = ?
            )
            "#,
        )
            .bind(i32::from(
                common.origin_webview_storage_may_contain_secrets(),
            ))
            .bind(now)
            .bind(common.id())
            .execute(&mut self.conn)
            .await
            .with_context(|| format!("failed to persist origin taint for app {}", common.id()))?;

        sqlx::query(
            r#"
            UPDATE sage_apps
            SET
                app_dir = ?,
                source_json = ?,
                granted_permissions_json = ?,
                wallet_scope_json = ?,
                active_snapshot_manifest_hash = ?,
                active_snapshot_dir = ?,
                pending_update_app_url = ?,
                pending_update_manifest_hash = ?,
                pending_update_manifest_json = ?,
                updated_at_ms = ?
            WHERE app_id = ?
            "#,
        )
            .bind(common.app_dir())
            .bind(serde_json::to_string(app.source())?)
            .bind(serde_json::to_string(common.granted_permissions())?)
            .bind(serde_json::to_string(common.wallet_scope())?)
            .bind(common.active_snapshot().manifest_hash())
            .bind(common.active_snapshot().snapshot_dir())
            .bind(app.pending_update().map(|p| p.app_url().to_string()))
            .bind(app.pending_update().map(|p| p.manifest_hash().to_string()))
            .bind(
                app.pending_update()
                    .map(|p| serde_json::to_string(p.manifest()))
                    .transpose()?,
            )
            .bind(now)
            .bind(common.id())
            .execute(&mut self.conn)
            .await
            .with_context(|| format!("failed to persist app {}", common.id()))?;

        Ok(())
    }

    pub(crate) async fn delete_user_app(&mut self, app_id: &str) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM sage_apps WHERE app_id = ?")
            .bind(app_id)
            .execute(&mut self.conn)
            .await
            .with_context(|| format!("failed to unregister app {app_id}"))?;
        Ok(())
    }

    pub(crate) async fn update_app_assignment(
        &mut self,
        app_id: &str,
        storage_id: i64,
        origin_row_id: i64,
    ) -> Result<()> {
        let now = crate::utils::unix_timestamp_ms();

        sqlx::query(
            r#"
            UPDATE sage_apps
            SET
                storage_id = ?,
                origin_row_id = ?,
                updated_at_ms = ?
            WHERE app_id = ?
            "#,
        )
            .bind(storage_id)
            .bind(origin_row_id)
            .bind(now)
            .bind(app_id)
            .execute(&mut self.conn)
            .await
            .with_context(|| format!("failed to update app assignment {app_id}"))?;

        Ok(())
    }

    pub(crate) async fn register_origin(
        &mut self,
        origin_id: &str,
        storage_id: i64,
    ) -> Result<i64> {
        let now = crate::utils::unix_timestamp_ms();

        let result = sqlx::query(
            r#"
            INSERT INTO sage_app_origins (
                origin_id,
                storage_id,
                may_contain_secrets,
                created_at_ms,
                updated_at_ms
            )
            VALUES (?, ?, 0, ?, ?)
            "#,
        )
            .bind(origin_id)
            .bind(storage_id)
            .bind(now)
            .bind(now)
            .execute(&mut self.conn)
            .await
            .with_context(|| format!("failed to insert origin {origin_id}"))?;

        Ok(result.last_insert_rowid())
    }
}

async fn list_user_app_ids_from_conn(conn: &mut SqliteConnection) -> Result<Vec<String>> {
    let rows = sqlx::query(
        r#"
        SELECT app_id
        FROM sage_apps
        ORDER BY app_id ASC
        "#,
    )
        .fetch_all(conn)
        .await
        .context("failed to list installed app ids")?;

    rows.into_iter()
        .map(|row| row.try_get("app_id"))
        .collect::<std::result::Result<Vec<_>, _>>()
        .context("failed to read installed app ids")
}

async fn load_user_app_optional_from_conn(
    conn: &mut SqliteConnection,
    app_id: &str,
) -> Result<Option<UserSageApp>> {
    let row = sqlx::query(load_user_app_sql())
        .bind(app_id)
        .fetch_optional(conn)
        .await
        .with_context(|| format!("failed to load app {app_id}"))?;

    row.map(row_to_user_app).transpose()
}

async fn load_user_app_from_conn(
    conn: &mut SqliteConnection,
    app_id: &str,
) -> Result<UserSageApp> {
    let row = sqlx::query(load_user_app_sql())
        .bind(app_id)
        .fetch_one(conn)
        .await
        .with_context(|| format!("failed to load app {app_id}"))?;

    row_to_user_app(row)
}

fn load_user_app_sql() -> &'static str {
    r#"
    SELECT
        apps.app_id,
        apps.app_dir,
        apps.source_json,
        apps.granted_permissions_json,
        apps.wallet_scope_json,
        apps.active_snapshot_manifest_hash,
        apps.active_snapshot_dir,
        apps.pending_update_app_url,
        apps.pending_update_manifest_hash,
        apps.pending_update_manifest_json,
        origins.origin_id,
        origins.may_contain_secrets,
        storages.storage_json
    FROM sage_apps apps
    INNER JOIN sage_app_origins origins
        ON origins.id = apps.origin_row_id
    INNER JOIN sage_app_storages storages
        ON storages.id = apps.storage_id
    WHERE apps.app_id = ?
    "#
}

fn row_to_user_app(row: SqliteRow) -> Result<UserSageApp> {
    let app_id: String = row.try_get("app_id")?;
    let app_dir: String = row.try_get("app_dir")?;
    let origin_id: String = row.try_get("origin_id")?;
    let may_contain_secrets: i64 = row.try_get("may_contain_secrets")?;

    let storage: SageAppStorage =
        serde_json::from_str(&row.try_get::<String, _>("storage_json")?)
            .context("failed to deserialize app storage")?;

    let source: UserSageAppSource =
        serde_json::from_str(&row.try_get::<String, _>("source_json")?)
            .context("failed to deserialize app source")?;

    let granted_permissions: SageGrantedPermissions =
        serde_json::from_str(&row.try_get::<String, _>("granted_permissions_json")?)
            .context("failed to deserialize granted permissions")?;

    let wallet_scope: SageAppWalletScope =
        serde_json::from_str(&row.try_get::<String, _>("wallet_scope_json")?)
            .context("failed to deserialize wallet scope")?;

    let active_snapshot = snapshot_from_row(&row)?;
    let pending_update = pending_update_from_row(&row)?;

    let common = SageAppCommon::from_persisted_parts(
        SageAppIdentity::new(app_id, origin_id, app_dir)?,
        granted_permissions,
        storage,
        may_contain_secrets != 0,
        active_snapshot,
        wallet_scope,
    )?;

    Ok(UserSageApp::load_persisted(common, source, pending_update))
}

async fn load_corrupted_user_app_from_conn(
    conn: &mut SqliteConnection,
    app_id: &str,
    error: anyhow::Error,
) -> Result<ListedSageApp> {
    let row = sqlx::query(
        r#"
        SELECT
            app_id,
            app_dir,
            source_json,
            active_snapshot_manifest_hash,
            active_snapshot_dir
        FROM sage_apps
        WHERE app_id = ?
        "#,
    )
        .bind(app_id)
        .fetch_one(conn)
        .await
        .with_context(|| format!("failed to load corrupted app fallback {app_id}"))?;

    let app_id: String = row.try_get("app_id")?;

    let app_dir: String = row
        .try_get::<Option<String>, _>("app_dir")?
        .unwrap_or_default();

    let source = row
        .try_get::<Option<String>, _>("source_json")?
        .and_then(|json| serde_json::from_str::<UserSageAppSource>(&json).ok());

    let snapshot_dir = row.try_get::<Option<String>, _>("active_snapshot_dir")?;

    let manifest_header = snapshot_dir
        .as_deref()
        .and_then(|dir| {
            let path = Path::new(dir).join(crate::types::MANIFEST_FILE_NAME);
            std::fs::read_to_string(path).ok()
        })
        .and_then(|text| serde_json::from_str::<serde_json::Value>(&text).ok())
        .and_then(|manifest| crate::types::parse_manifest_header_v0_from_value(manifest).ok());

    let icon = manifest_header
        .as_ref()
        .and_then(|header| header.icon.as_deref())
        .and_then(|icon_path| snapshot_dir.as_deref().map(|dir| Path::new(dir).join(icon_path)))
        .and_then(|path| SageAppIconView::from_file_path(&path));

    Ok(ListedSageApp::Corrupted(
        CorruptedInstalledSageApp::new(app_id, app_dir, error.to_string())
            .with_manifest_header(manifest_header)
            .with_source(source)
            .with_icon(icon),
    ))
}

fn read_snapshot_manifest(snapshot_dir: &str) -> Result<SageAppPackageManifest> {
    let manifest_path = Path::new(snapshot_dir).join(crate::types::MANIFEST_FILE_NAME);

    let text = std::fs::read_to_string(&manifest_path)
        .with_context(|| format!("failed to read snapshot manifest {}", manifest_path.display()))?;

    serde_json::from_str(&text)
        .with_context(|| format!("failed to parse snapshot manifest {}", manifest_path.display()))
}

fn snapshot_from_row(row: &SqliteRow) -> Result<SageAppSnapshot> {
    let manifest_hash: String = row.try_get("active_snapshot_manifest_hash")?;
    let snapshot_dir: String = row.try_get("active_snapshot_dir")?;
    let manifest = read_snapshot_manifest(&snapshot_dir)?;

    SageAppSnapshot::new(manifest_hash, snapshot_dir, manifest)
}

fn pending_update_from_row(row: &SqliteRow) -> Result<Option<UserSageAppPendingUpdate>> {
    let Some(app_url) = row.try_get::<Option<String>, _>("pending_update_app_url")? else {
        return Ok(None);
    };

    let manifest_hash: String = row
        .try_get::<Option<String>, _>("pending_update_manifest_hash")?
        .context("pending update app url exists without manifest hash")?;

    let manifest_json: String = row
        .try_get::<Option<String>, _>("pending_update_manifest_json")?
        .context("pending update app url exists without manifest")?;

    Ok(Some(UserSageAppPendingUpdate::new(
        SageAppUrl::parse(&app_url)?,
        manifest_hash,
        serde_json::from_str(&manifest_json)
            .context("failed to deserialize pending update manifest")?,
    )))
}
