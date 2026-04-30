use serde::{Deserialize, Serialize};
use specta::Type;
use crate::types::{
    CorruptedInstalledSageApp, ListedSageApp, SageApp, SharedSageApp, UserSageApp,
    UserSageAppSource,
};
use crate::types::app::view::system_apps::SystemSageAppView;
use crate::types::app::view::common::SageAppCommonView;

#[derive(Debug, Clone, Serialize, Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[allow(clippy::large_enum_variant)]
pub enum ListedSageAppView {
    User(UserSageAppView),
    System(SystemSageAppView),
    Corrupted(CorruptedInstalledSageApp),
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub enum SageAppView {
    System(SystemSageAppView),
    User(UserSageAppView),
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct UserSageAppView {
    common: SageAppCommonView,
    source: UserSageAppSource,
}

impl From<&SharedSageApp> for SageAppView {
    fn from(app: &SharedSageApp) -> Self {
        app.with(|app| match app {
            SageApp::User(app) => SageAppView::User(app.into()),
            SageApp::System(app) => SageAppView::System(app.into()),
        })
    }
}

impl From<SharedSageApp> for SageAppView {
    fn from(app: SharedSageApp) -> Self {
        (&app).into()
    }
}

impl From<&UserSageApp> for UserSageAppView {
    fn from(app: &UserSageApp) -> Self {
        Self {
            common: app.common().into(),
            source: app.source().clone(),
        }
    }
}

impl From<UserSageApp> for UserSageAppView {
    fn from(app: UserSageApp) -> Self {
        (&app).into()
    }
}

impl From<&ListedSageApp> for ListedSageAppView {
    fn from(value: &ListedSageApp) -> Self {
        match value {
            ListedSageApp::User(app) => ListedSageAppView::User(app.into()),
            ListedSageApp::System(app) => ListedSageAppView::System(app.into()),
            ListedSageApp::Corrupted(app) => ListedSageAppView::Corrupted(app.clone()),
        }
    }
}
