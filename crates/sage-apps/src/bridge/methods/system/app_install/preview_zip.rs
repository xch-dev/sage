use std::fs;
use std::path::Path;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use specta::Type;
use uuid::Uuid;

use crate::{
    BridgeApprovalRequestResult, BridgeContext, BridgeHandleResult, BridgeMethod,
    BridgeMethodCapability, BridgeMethodHandleError, BridgeTools, RustBridgeRequest,
    SageAppCompatibility, SageAppPackageManifestPreview, SystemBridgeCapability,
    detect_package_root, parse_required_params, read_manifest_preview, unzip_to_dir,
};

#[derive(Debug, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AppInstallPreviewZipParams {
    zip_path: String,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AppInstallPreviewZipResult {
    preview: SageAppPackageManifestPreview,
    compatibility: SageAppCompatibility,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct AppInstallPreviewZip;

#[async_trait]
impl BridgeMethod for AppInstallPreviewZip {
    fn name(&self) -> &'static str {
        "appInstall.previewZip"
    }

    fn capability(&self) -> BridgeMethodCapability {
        BridgeMethodCapability::system(SystemBridgeCapability::AppInstallPreview)
    }

    fn approval_request(
        &self,
        _ctx: BridgeContext<'_>,
        _request: &RustBridgeRequest,
    ) -> BridgeApprovalRequestResult {
        Ok(None)
    }

    async fn handle(
        &self,
        _ctx: BridgeContext<'_>,
        tools: BridgeTools<'_>,
        request: &RustBridgeRequest,
    ) -> BridgeHandleResult {
        let params: AppInstallPreviewZipParams = parse_required_params(self, request)?;

        let preview =
            preview_manifest(&params.zip_path).map_err(BridgeMethodHandleError::internal_error)?;

        let compatibility = SageAppCompatibility::for_app(
            tools.app_handle,
            &preview.manifest_header().sage_version,
        );

        Ok(Box::new(AppInstallPreviewZipResult {
            preview,
            compatibility,
        }))
    }
}

fn preview_manifest(zip_path: &String) -> Result<SageAppPackageManifestPreview, String> {
    let unpack_dir = std::env::temp_dir().join(format!(".sage-preview-{}", Uuid::new_v4()));

    let result = (|| -> anyhow::Result<SageAppPackageManifestPreview> {
        unzip_to_dir(Path::new(&zip_path), &unpack_dir)?;
        let package_root = detect_package_root(&unpack_dir)?;
        let manifest = read_manifest_preview(&package_root)?;

        Ok(manifest)
    })();

    let _ = fs::remove_dir_all(&unpack_dir);

    result.map_err(|err| format!("failed to preview app zip {zip_path}: {err}"))
}
