use serde::{Serialize};
use specta::Type;
use url::Url;
use crate::types::app::view::permission::SageGrantedPermissionsView;
use crate::types::app::view::snapshot::SageAppSnapshotView;
use crate::types::{SageAppCommon, SageAppIdentity, SageAppPackageManifest, SageAppPackageManifestPreview, SageAppUrl, SageAppWalletScope};

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SageAppIdentityView {
    id: String,
    origin_id: String,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SageAppCommonView {
    identity: SageAppIdentityView,
    granted_permissions: SageGrantedPermissionsView,
    wallet_scope: SageAppWalletScope,
    active_snapshot: SageAppSnapshotView,
    icon: Option<SageAppIconView>,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SageAppIconView {
    mime: String,
    bytes: Vec<u8>,
}

impl From<&SageAppCommon> for SageAppCommonView {
    fn from(common: &SageAppCommon) -> Self {
        Self {
            identity: common.identity().into(),
            active_snapshot: common.active_snapshot().into(),
            granted_permissions: common.granted_permissions().into(),
            wallet_scope: common.wallet_scope().clone(),
            icon: SageAppIconView::from_common(common),
        }
    }
}

impl From<&SageAppIdentity> for SageAppIdentityView {
    fn from(value: &SageAppIdentity) -> Self {
        Self {
            id: value.id().to_string(),
            origin_id: value.origin_id().to_string()
        }
    }
}

impl SageAppIconView {
    pub(crate) fn from_common(common: &SageAppCommon) -> Option<Self> {
        let icon_path = common.active_snapshot().manifest().icon()?;
        Self::from_common_file(common, icon_path)
    }

    pub(crate) fn author_avatar_from_common(common: &SageAppCommon) -> Option<Self> {
        let avatar_path = common
            .active_snapshot()
            .manifest()
            .author()?
            .avatar()?;

        Self::from_common_file(common, avatar_path)
    }

    fn from_common_file(common: &SageAppCommon, path: &str) -> Option<Self> {
        let file_path = common
            .active_snapshot()
            .resolve_file_path(path)
            .ok()?;

        Self::from_file_path(&file_path)
    }

    pub(crate) fn from_file_path(file_path: &std::path::Path) -> Option<Self> {
        let bytes = std::fs::read(file_path).ok()?;
        let mime = mime_guess::from_path(file_path)
            .first_or_octet_stream()
            .essence_str()
            .to_string();

        Some(Self { mime, bytes })
    }

    pub(crate) async fn from_url_manifest(
        base: &SageAppUrl,
        manifest: &SageAppPackageManifest,
    ) -> Option<Self> {
        let icon_path = manifest.icon()?;
        Self::from_url(base, icon_path).await
    }

    pub async fn from_url_preview(
        base: &SageAppUrl,
        preview: &SageAppPackageManifestPreview,
    ) -> Option<Self> {
        match preview {
            SageAppPackageManifestPreview::Full { manifest } => {
                Self::from_url_manifest(base, manifest).await
            }
            SageAppPackageManifestPreview::Partial { manifest_header, .. } => {
                let icon_path = manifest_header.icon.as_deref()?;

                Self::from_url(base, icon_path).await
            }
        }
    }

    async fn from_url(
        base: &SageAppUrl,
        icon_path: &str,
    ) -> Option<Self> {
        let base_url = Url::parse(base.as_str()).ok()?;
        let resolved = base_url.join(icon_path).ok()?;

        let resp = reqwest::get(resolved).await.ok()?;
        if !resp.status().is_success() {
            return None;
        }

        let mime = resp
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("application/octet-stream")
            .to_string();

        let bytes = resp.bytes().await.ok()?;

        Some(Self {
            mime,
            bytes: bytes.to_vec(),
        })
    }
}
