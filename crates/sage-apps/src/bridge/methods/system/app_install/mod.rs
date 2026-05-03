mod preview_url;
mod preview_zip;
mod install_url;
mod install_zip;

use serde::Serialize;
use specta::Type;
pub(crate) use preview_url::{AppInstallPreviewUrl, AppInstallPreviewUrlParams};
pub(crate) use preview_zip::{AppInstallPreviewZip, AppInstallPreviewZipParams};
pub(crate) use install_url::{AppInstallInstallUrl, AppInstallInstallUrlParams};
pub(crate) use install_zip::{AppInstallInstallZip, AppInstallInstallZipParams};
use crate::types::UserSageAppView;

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
