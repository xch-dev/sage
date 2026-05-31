mod install_url;
mod install_zip;
mod preview_url;
mod preview_zip;

use crate::types::UserSageAppView;
pub(crate) use install_url::{AppInstallInstallUrl, AppInstallInstallUrlParams};
pub(crate) use install_zip::{AppInstallInstallZip, AppInstallInstallZipParams};
pub(crate) use preview_url::{AppInstallPreviewUrl, AppInstallPreviewUrlParams};
pub(crate) use preview_zip::{AppInstallPreviewZip, AppInstallPreviewZipParams};
use serde::Serialize;
use specta::Type;

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
