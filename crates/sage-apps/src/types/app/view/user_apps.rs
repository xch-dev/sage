use crate::types::app::view::common::SageAppCommonView;
use crate::types::app::view::preview::UserSageAppPendingUpdateView;
use crate::types::app::view::system_apps::SystemSageAppView;
use crate::types::{
    CorruptedInstalledSageApp, ListedSageApp, SageApp, SharedSageApp, UserSageApp,
    UserSageAppSource,
};
use serde::Serialize;
use specta::Type;

#[derive(Debug, Clone, Serialize, Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[allow(clippy::large_enum_variant)]
pub enum ListedSageAppView {
    User(UserSageAppView),
    System(SystemSageAppView),
    Corrupted(CorruptedInstalledSageApp),
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, Serialize, Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum SageAppView {
    System(SystemSageAppView),
    User(UserSageAppView),
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct UserSageAppView {
    common: SageAppCommonView,
    source: UserSageAppSource,

    #[serde(skip_serializing_if = "Option::is_none")]
    pending_update: Option<UserSageAppPendingUpdateView>,
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
            pending_update: app.pending_update().map(Into::into),
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
