#[derive(Debug, Clone)]
pub struct StorageCleanupRuntimeTarget {
    pub app_id: String,
    pub origin_id: String,
    pub storage: crate::types::SageAppStorage,
}
