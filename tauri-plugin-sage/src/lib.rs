use tauri::{
    Manager, Runtime,
    plugin::{Builder, TauriPlugin},
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
use desktop::Sage;
#[cfg(mobile)]
use mobile::Sage;

/// Extensions to [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`] to access the sage APIs.
pub trait SageExt<R: Runtime> {
    fn sage(&self) -> &Sage<R>;
}

impl<R: Runtime, T: Manager<R>> crate::SageExt<R> for T {
    fn sage(&self) -> &Sage<R> {
        self.state::<Sage<R>>().inner()
    }
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("sage")
        .invoke_handler(tauri::generate_handler![
            commands::is_ndef_available,
            commands::get_ndef_payloads
        ])
        .setup(|app, api| {
            #[cfg(mobile)]
            let sage = mobile::init(app, api)?;
            #[cfg(desktop)]
            let sage = desktop::init(app, api)?;
            app.manage(sage);
            Ok(())
        })
        .build()
}
