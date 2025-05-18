use tauri::{command, AppHandle, Runtime};

use crate::models::*;
use crate::Result;
use crate::SageExt;

#[command]
pub(crate) async fn ping<R: Runtime>(
    app: AppHandle<R>,
    payload: PingRequest,
) -> Result<PingResponse> {
    app.sage().ping(payload)
}
