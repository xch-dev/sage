mod install_url;
mod install_zip;
mod preview_url;
mod preview_zip;

pub(crate) use install_url::*;
pub(crate) use install_zip::*;
pub(crate) use preview_url::*;
pub(crate) use preview_zip::*;

use serde::Serialize;
use specta::Type;

use crate::{SystemBridgeCapability, SystemRuntimeEvent, UserSageAppView};

#[derive(Debug, Clone, Copy, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AppInstallDownloadProgressEvent {
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
}

impl SystemRuntimeEvent for AppInstallDownloadProgressEvent {
    const TYPE: &'static str = "appInstall.downloadProgress";
    const REQUIRED_CAPABILITY: SystemBridgeCapability = SystemBridgeCapability::AppInstallApply;
}

#[derive(Debug, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AppInstallInstallResult {
    app: UserSageAppView,
}

impl AppInstallInstallResult {
    pub fn new(app: UserSageAppView) -> Self {
        Self { app }
    }
}
