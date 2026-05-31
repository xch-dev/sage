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
