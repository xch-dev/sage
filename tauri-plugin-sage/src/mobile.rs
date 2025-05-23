use serde::de::DeserializeOwned;
use tauri::{
    plugin::{PluginApi, PluginHandle},
    AppHandle, Runtime,
};

use crate::models::*;

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_sage);

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

    pub fn scan_tangem_card(&self) -> crate::Result<ScanTangemCardResponse> {
        self.0
            .run_mobile_plugin("scanTangemCard", ScanTangemCardRequest {})
            .map_err(Into::into)
    }
}
