pub mod install;
pub mod manifest;
mod mutation;
pub mod package;
pub mod registry;
mod scope;
pub mod snapshot;
pub mod storage;
pub mod uninstall;
pub mod update;

pub use manifest::*;
pub use package::*;
pub use registry::*;
pub use snapshot::*;
pub use storage::*;

pub use update::start_background_app_update_checker;

pub(crate) use mutation::AppMutationManager;
pub(crate) use scope::ensure_app_is_enabled_for_scope;
