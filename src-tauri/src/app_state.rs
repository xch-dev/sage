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
    pub unit: Unit,
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
            unit: XCH.clone(),
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

    #[allow(clippy::needless_pass_by_value)]
    pub fn parse_address(&self, input: String) -> Result<Bytes32> {
        let (puzzle_hash, prefix) = decode_address(&input)?;

        if prefix != self.network().address_prefix {
            return Err(Error {
                kind: ErrorKind::Api,
                reason: format!("Wrong address prefix: {prefix}"),
            });
        }

        Ok(puzzle_hash.into())
    }

    #[allow(clippy::needless_pass_by_value)]
    pub fn parse_amount(&self, input: Amount) -> Result<u64> {
        let Some(amount) = input.to_mojos(self.unit.decimals) else {
            return Err(Error {
                kind: ErrorKind::Api,
                reason: format!("Invalid amount: {input}"),
            });
        };

        Ok(amount)
    }
}
