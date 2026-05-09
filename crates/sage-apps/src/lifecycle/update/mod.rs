pub mod commands;
pub mod permissions;
pub mod scope;
pub mod types;
pub mod logic;
mod background;

pub use background::start_background_app_update_checker;
