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
use desktop::SafeAreaInsets;
#[cfg(mobile)]
use mobile::SafeAreaInsets;

/// Extensions to [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`] to access the safe-area-insets APIs.
pub trait SafeAreaInsetsExt<R: Runtime> {
  fn safe_area_insets(&self) -> &SafeAreaInsets<R>;
}

impl<R: Runtime, T: Manager<R>> crate::SafeAreaInsetsExt<R> for T {
  fn safe_area_insets(&self) -> &SafeAreaInsets<R> {
    self.state::<SafeAreaInsets<R>>().inner()
  }
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
  Builder::new("safe-area-insets")
    .invoke_handler(tauri::generate_handler![commands::get_insets])
    .setup(|app, api| {
      #[cfg(mobile)]
      let safe_area_insets = mobile::init(app, api)?;
      #[cfg(desktop)]
      let safe_area_insets = desktop::init(app, api)?;
      app.manage(safe_area_insets);
      Ok(())
    })
    .build()
}
