use serde::de::DeserializeOwned;
use tauri::{plugin::PluginApi, AppHandle, Runtime};

use crate::models::*;

pub fn init<R: Runtime, C: DeserializeOwned>(
    app: &AppHandle<R>,
    _api: PluginApi<R, C>,
) -> crate::Result<Sage<R>> {
    Ok(Sage(app.clone()))
}

/// Access to the sage APIs.
pub struct Sage<R: Runtime>(AppHandle<R>);

impl<R: Runtime> Sage<R> {
    pub fn is_ndef_available(&self) -> crate::Result<IsNdefAvailableResponse> {
        Ok(IsNdefAvailableResponse { available: false })
    }

    pub fn get_ndef_payloads(&self) -> crate::Result<GetNdefPayloadsResponse> {
        Ok(GetNdefPayloadsResponse { payloads: vec![] })
    }
}
