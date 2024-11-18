use std::{
    ops::{Deref, DerefMut},
    path::Path,
};

use chia::protocol::Bytes32;
use chia_wallet_sdk::decode_address;
use sage::Sage;
use sage_api::{Amount, SyncEvent as ApiEvent, Unit, XCH};
use sage_wallet::SyncEvent;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;

use crate::error::{Error, ErrorKind, Result};

pub type AppState = Mutex<AppStateInner>;

pub struct AppStateInner {
    pub app_handle: AppHandle,
    pub initialized: bool,
    pub sage: Sage,
}

impl Deref for AppStateInner {
    type Target = Sage;

    fn deref(&self) -> &Self::Target {
        &self.sage
    }
}

impl DerefMut for AppStateInner {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.sage
    }
}

impl AppStateInner {
    pub fn new(app_handle: AppHandle, path: &Path) -> Self {
        Self {
            app_handle,
            initialized: false,
            sage: Sage::new(path),
        }
    }

    pub async fn initialize(&mut self) -> Result<()> {
        if self.initialized {
            return Ok(());
        }

        self.initialized = true;

        let mut receiver = self.sage.initialize().await?;
        let app_handle = self.app_handle.clone();

        tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                let event = match event {
                    SyncEvent::Start(ip) => ApiEvent::Start { ip: ip.to_string() },
                    SyncEvent::Stop => ApiEvent::Stop,
                    SyncEvent::Subscribed => ApiEvent::Subscribed,
                    SyncEvent::Derivation => ApiEvent::Derivation,
                    // TODO: New event?
                    SyncEvent::CoinState | SyncEvent::Transaction => ApiEvent::CoinState,
                    SyncEvent::PuzzleBatchSynced => ApiEvent::PuzzleBatchSynced,
                    SyncEvent::CatInfo => ApiEvent::CatInfo,
                    SyncEvent::DidInfo => ApiEvent::DidInfo,
                    SyncEvent::NftData => ApiEvent::NftData,
                };
                if app_handle.emit("sync-event", event).is_err() {
                    break;
                }
            }

            Result::Ok(())
        });

        Ok(())
    }
}
