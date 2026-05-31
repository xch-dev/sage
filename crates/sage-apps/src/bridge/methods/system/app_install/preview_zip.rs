use std::fs;
use std::path::Path;

use serde::Deserialize;
use specta::Type;
use uuid::Uuid;

use crate::{
    BridgeApprovalRequestResult, BridgeContext, BridgeHandleResult, BridgeMethodCapability,
    BridgeMethodHandleError, BridgeMethodHandler, BridgeTools, RustBridgeRequest,
    SageAppPackageManifest, SystemBridgeCapability, bridge_result, detect_package_root,
    parse_required_params, read_manifest, unzip_to_dir,
};

#[derive(Debug, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AppInstallPreviewZipParams {
    zip_path: String,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct AppInstallPreviewZip;

impl BridgeMethodHandler for AppInstallPreviewZip {
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
        _tools: BridgeTools<'_>,
        request: &RustBridgeRequest,
    ) -> BridgeHandleResult {
        let params: AppInstallPreviewZipParams = parse_required_params(self, request)?;

        let manifest =
            preview_manifest(&params.zip_path).map_err(BridgeMethodHandleError::internal_error)?;

        bridge_result(manifest)
    }
}

fn preview_manifest(zip_path: &String) -> Result<SageAppPackageManifest, String> {
    let unpack_dir = std::env::temp_dir().join(format!(".sage-preview-{}", Uuid::new_v4()));

    let result = (|| -> anyhow::Result<SageAppPackageManifest> {
        unzip_to_dir(Path::new(&zip_path), &unpack_dir)?;
        let package_root = detect_package_root(&unpack_dir)?;
        let manifest = read_manifest(&package_root)?;

        Ok(manifest)
    })();

    let _ = fs::remove_dir_all(&unpack_dir);

    result.map_err(|err| format!("failed to preview app zip {zip_path}: {err}"))
}
