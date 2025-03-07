use serde::de::DeserializeOwned;
use tauri::{plugin::PluginApi, AppHandle, Runtime};

use crate::models::*;

pub fn init<R: Runtime, C: DeserializeOwned>(
  app: &AppHandle<R>,
  _api: PluginApi<R, C>,
) -> crate::Result<NfcDebug<R>> {
  Ok(NfcDebug(app.clone()))
}

/// Access to the nfc-debug APIs.
pub struct NfcDebug<R: Runtime>(AppHandle<R>);

impl<R: Runtime> NfcDebug<R> {
  pub fn ping(&self, payload: PingRequest) -> crate::Result<PingResponse> {
    Ok(PingResponse {
      value: payload.value,
    })
  }
}
