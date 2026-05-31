use crate::{SageApp, SageAppStorage, SageGrantedPermissions};

#[derive(Debug)]
pub(crate) struct AppMutationDraft {
    app: SageApp,
}

impl AppMutationDraft {
    pub(crate) fn new(app: SageApp) -> Self {
        Self { app }
    }

    pub(crate) fn app(&self) -> &SageApp {
        &self.app
    }

    pub(crate) fn app_mut(&mut self) -> &mut SageApp {
        &mut self.app
    }

    pub(crate) fn replace_storage_and_origin(
        &mut self,
        storage: SageAppStorage,
        origin_id: impl Into<String>,
        origin_tainted: bool,
    ) -> anyhow::Result<()> {
        self.app
            .common_mut()
            .replace_storage_and_origin(storage, origin_id, origin_tainted)
    }

    pub(crate) fn mark_origin_webview_storage_may_contain_secrets(&mut self) -> anyhow::Result<()> {
        self.app
            .common_mut()
            .mark_origin_webview_storage_may_contain_secrets()
    }

    pub(crate) fn update_permissions(
        &mut self,
        granted_permissions: &SageGrantedPermissions,
    ) -> anyhow::Result<()> {
        self.app
            .common_mut()
            .update_permissions(granted_permissions)
    }
}
