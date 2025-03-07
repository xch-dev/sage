use tauri::{
  plugin::{Builder, TauriPlugin},
  Manager, Runtime,
};

pub use models::*;

#[cfg(desktop)]
mod desktop;
#[cfg(mobile)]
mod mobile;

mod commands;
mod error;
mod models;

pub use error::{Error, Result};

#[cfg(desktop)]
use desktop::NfcDebug;
#[cfg(mobile)]
use mobile::NfcDebug;

/// Extensions to [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`] to access the nfc-debug APIs.
pub trait NfcDebugExt<R: Runtime> {
  fn nfc_debug(&self) -> &NfcDebug<R>;
}

impl<R: Runtime, T: Manager<R>> crate::NfcDebugExt<R> for T {
  fn nfc_debug(&self) -> &NfcDebug<R> {
    self.state::<NfcDebug<R>>().inner()
  }
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
  Builder::new("nfc-debug")
    .invoke_handler(tauri::generate_handler![commands::ping])
    .setup(|app, api| {
      #[cfg(mobile)]
      let nfc_debug = mobile::init(app, api)?;
      #[cfg(desktop)]
      let nfc_debug = desktop::init(app, api)?;
      app.manage(nfc_debug);
      Ok(())
    })
    .build()
}
