use std::{
    fs,
    ops::Deref,
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
};

use chia::{
    bls::{master_to_wallet_unhardened_intermediate, PublicKey},
    protocol::Bytes32,
};
use sage_api::{Unit, TXCH, XCH};
use sage_config::Assets;
use sage_database::Database;
use sage_wallet::Wallet;
use sqlx::ConnectOptions;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
    SqlitePool,
};
use tauri::async_runtime::Sender;
use tokio::{
    sync::{mpsc, oneshot, Mutex},
    task::JoinHandle,
};
use tracing::error;

use crate::Result;

pub struct WalletState {
    pub inner: Arc<Wallet>,
    pub unit: Unit,
    pub assets: Arc<Mutex<Assets>>,
    pub saver: Sender<oneshot::Sender<()>>,
    save_task: JoinHandle<()>,
}

impl WalletState {
    pub async fn open(
        wallet_path: PathBuf,
        network_id: String,
        genesis_challenge: Bytes32,
        master_pk: PublicKey,
    ) -> Result<Self> {
        let intermediate_pk = master_to_wallet_unhardened_intermediate(&master_pk);
        let fingerprint = master_pk.get_fingerprint();
        let db_path = Self::db_path(&wallet_path, &network_id);
        let assets_path = Self::assets_path(&wallet_path, &network_id);

        fs::create_dir_all(wallet_path.join("db"))?;
        fs::create_dir_all(wallet_path.join("assets"))?;

        let mut pool = Self::connect_db(&db_path).await?;

        // TODO: Remove this before out of beta.
        if let Err(error) = sqlx::migrate!("../migrations").run(&pool).await {
            error!("Error migrating database, dropping database: {error:?}");
            Self::delete_db(pool, &db_path).await?;
            pool = Self::connect_db(&db_path).await?;
            sqlx::migrate!("../migrations").run(&pool).await?;
        }

        let assets = if assets_path.try_exists()? {
            serde_json::from_str(&fs::read_to_string(&assets_path)?)?
        } else {
            let assets = Assets::default();
            Self::save_assets(&assets_path, &assets)?;
            assets
        };

        let assets = Arc::new(Mutex::new(assets));

        let (sender, mut receiver) = mpsc::channel::<oneshot::Sender<()>>(1);

        let save_task = tokio::spawn({
            let assets = assets.clone();
            async move {
                while let Some(callback) = receiver.recv().await {
                    if let Err(error) = Self::save_assets(&assets_path, &*assets.lock().await) {
                        error!("Error saving assets: {error:?}");
                    }
                    callback.send(()).ok();
                }
            }
        });

        Ok(Self {
            inner: Arc::new(Wallet::new(
                Database::new(pool),
                fingerprint,
                intermediate_pk,
                genesis_challenge,
            )),
            unit: match network_id.as_str() {
                "mainnet" => XCH.clone(),
                _ => TXCH.clone(),
            },
            assets,
            saver: sender,
            save_task,
        })
    }

    fn db_path(path: &Path, network_id: &str) -> PathBuf {
        path.join("db").join(format!("{network_id}.sqlite"))
    }

    fn assets_path(path: &Path, network_id: &str) -> PathBuf {
        path.join("assets").join(format!("{network_id}.json"))
    }

    async fn connect_db(db_path: &Path) -> Result<SqlitePool> {
        Ok(SqlitePoolOptions::new()
            .connect_with(
                SqliteConnectOptions::from_str(&format!(
                    "sqlite://{}?mode=rwc",
                    db_path.display()
                ))?
                .journal_mode(SqliteJournalMode::Wal)
                .log_statements(log::LevelFilter::Trace),
            )
            .await?)
    }

    async fn delete_db(db: SqlitePool, db_path: &Path) -> Result<()> {
        db.close().await;
        drop(db);
        fs::remove_file(db_path)?;
        Ok(())
    }

    fn save_assets(assets_path: &Path, assets: &Assets) -> Result<()> {
        fs::write(assets_path, serde_json::to_string_pretty(assets)?)?;
        Ok(())
    }
}

impl Deref for WalletState {
    type Target = Wallet;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Drop for WalletState {
    fn drop(&mut self) {
        self.save_task.abort();
    }
}
