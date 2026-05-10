mod background;
pub mod commands;
pub mod permissions;
pub mod scope;
pub mod types;
mod check;
mod apply;

pub use background::start_background_app_update_checker;
pub(crate) use apply::apply_app_update_inner;
pub(crate) use check::check_app_update_inner;
