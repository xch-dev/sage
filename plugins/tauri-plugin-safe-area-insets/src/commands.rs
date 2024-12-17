use tauri::{command, AppHandle, Runtime};

use crate::models::*;
use crate::Result;
use crate::SafeAreaInsetsExt;

#[command]
pub(crate) async fn get_insets<R: Runtime>(app: AppHandle<R>) -> Result<Insets> {
    app.safe_area_insets().get_insets()
}
