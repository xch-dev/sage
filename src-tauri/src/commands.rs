use specta::specta;
use tauri::{command, State};

use crate::{app_state::AppState, error::Result};

mod actions;
mod data;
mod keys;
mod settings;
mod transactions;

pub use actions::*;
pub use data::*;
pub use keys::*;
pub use settings::*;
pub use transactions::*;

#[command]
#[specta]
pub async fn initialize(state: State<'_, AppState>) -> Result<()> {
    state.lock().await.initialize().await
}
