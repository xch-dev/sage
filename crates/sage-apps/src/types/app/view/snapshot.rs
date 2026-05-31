use serde::Serialize;
use specta::Type;

use crate::types::{SageAppPackageManifest, SageAppSnapshot};

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SageAppSnapshotView {
    manifest: SageAppPackageManifest,
}

impl From<&SageAppSnapshot> for SageAppSnapshotView {
    fn from(value: &SageAppSnapshot) -> Self {
        Self {
            manifest: value.manifest().clone(),
        }
    }
}
