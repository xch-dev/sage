use tauri::{AppHandle, Runtime, command};

use crate::Result;
use crate::SageExt;
use crate::models::*;

#[command]
pub(crate) async fn is_ndef_available<R: Runtime>(
    app: AppHandle<R>,
) -> Result<IsNdefAvailableResponse> {
    app.sage().is_ndef_available()
}

#[command]
pub(crate) async fn get_ndef_payloads<R: Runtime>(
    app: AppHandle<R>,
) -> Result<GetNdefPayloadsResponse> {
    app.sage().get_ndef_payloads()
}
