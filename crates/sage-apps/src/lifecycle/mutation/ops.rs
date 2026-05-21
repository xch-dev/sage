use crate::lifecycle::mutation::AppMutationDraft;
use crate::types::{
    SageAppSnapshot, SageAppStorage, SageGrantedPermissions, UserSageAppPendingUpdate,
};

impl AppMutationDraft {
    pub(crate) fn apply_update(
        &mut self,
        pending: &UserSageAppPendingUpdate,
        granted_permissions: SageGrantedPermissions,
        snapshot: SageAppSnapshot,
    ) -> anyhow::Result<()> {
        self.app_mut()
            .apply_update(pending, granted_permissions, snapshot)
    }

    pub(crate) fn update_permissions(
        &mut self,
        granted_permissions: &SageGrantedPermissions,
    ) -> anyhow::Result<()> {
        self.app_mut()
            .common_mut()
            .update_permissions(granted_permissions)
    }

    pub(crate) fn rotate_resources(
        &mut self,
        storage: SageAppStorage,
        origin_id: impl Into<String>,
    ) -> anyhow::Result<()> {
        self.replace_storage_and_origin(storage, origin_id, false)
    }
}
