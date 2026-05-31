use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri_plugin_dialog::DialogExt;

use crate::{BridgeApprovalRequestResult, BridgeContext, BridgeHandleResult, BridgeMethod, BridgeMethodCapability, BridgeTools, parse_required_params, RustBridgeRequest, SystemBridgeCapability};

#[derive(Debug, Clone, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct FileSystemSelectFileFilter {
    name: String,
    extensions: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct FileSystemSelectFileParams {
    #[serde(default)]
    title: Option<String>,

    #[serde(default)]
    filters: Vec<FileSystemSelectFileFilter>,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct FileSystemSelectFileResult {
    path: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct FileSystemSelectFile;

#[async_trait]
impl BridgeMethod for FileSystemSelectFile {
    fn name(&self) -> &'static str {
        "fileSystem.selectFile"
    }

    fn capability(&self) -> BridgeMethodCapability {
        BridgeMethodCapability::system(SystemBridgeCapability::FileSystemSelectFile)
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
        let params: FileSystemSelectFileParams = parse_required_params(self, request)?;

        let mut dialog = tools.app_handle.dialog().file();

        if let Some(title) = params.title {
            dialog = dialog.set_title(title);
        }

        for filter in params.filters {
            let extensions = filter
                .extensions
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>();

            dialog = dialog.add_filter(filter.name, &extensions);
        }

        let selected = dialog.blocking_pick_file().map(|path| path.to_string());

        Ok(Box::new(FileSystemSelectFileResult { path: selected }))
    }
}
