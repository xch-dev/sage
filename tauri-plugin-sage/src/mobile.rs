use serde::de::DeserializeOwned;
use tauri::{
    AppHandle, Runtime,
    plugin::{PluginApi, PluginHandle},
};

use crate::models::*;

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_sage);

#[cfg(target_os = "ios")]
mod ios {
    use std::ffi::c_void;

    use swift_rs::{SRString, swift};

    swift!(pub fn sage_register_webview(webview: *const c_void, label: &SRString));
}

#[cfg(target_os = "ios")]
pub(crate) fn register_webview(webview: *const std::ffi::c_void, label: &str) {
    let label = swift_rs::SRString::from(label);
    unsafe { ios::sage_register_webview(webview, &label) };
}

// initializes the Kotlin or Swift plugin classes
pub fn init<R: Runtime, C: DeserializeOwned>(
    _app: &AppHandle<R>,
    api: PluginApi<R, C>,
) -> crate::Result<Sage<R>> {
    #[cfg(target_os = "android")]
    let handle = api.register_android_plugin("com.rigidnetwork.sage_plugin", "SagePlugin")?;
    #[cfg(target_os = "ios")]
    let handle = api.register_ios_plugin(init_plugin_sage)?;
    Ok(Sage(handle))
}

/// Access to the sage APIs.
pub struct Sage<R: Runtime>(PluginHandle<R>);

impl<R: Runtime> Sage<R> {
    pub fn is_ndef_available(&self) -> crate::Result<IsNdefAvailableResponse> {
        self.0
            .run_mobile_plugin("isNdefAvailable", IsNdefAvailableRequest {})
            .map_err(Into::into)
    }

    pub fn get_ndef_payloads(&self) -> crate::Result<GetNdefPayloadsResponse> {
        self.0
            .run_mobile_plugin("getNdefPayloads", GetNdefPayloadsRequest {})
            .map_err(Into::into)
    }

    pub fn set_webview_bounds(
        &self,
        request: SetWebviewBoundsRequest,
    ) -> crate::Result<SetWebviewBoundsResponse> {
        self.0
            .run_mobile_plugin("setWebviewBounds", request)
            .map_err(Into::into)
    }

    pub fn snapshot_webview(
        &self,
        request: SnapshotWebviewRequest,
    ) -> crate::Result<SnapshotWebviewResponse> {
        self.0
            .run_mobile_plugin("snapshotWebview", request)
            .map_err(Into::into)
    }
}
