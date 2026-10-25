//! XCHandles resolution.
//!
//! Handle data comes from the XCHandles listener API, but the API is not trusted. Every
//! lookup is checked against the chain through Sage's peers before a handle is treated as
//! verified: the slot must descend from the trusted registry singleton and be unspent, and
//! the resolved address is derived from the resolved singleton's latest on-chain spend.

mod api;
mod cache;
mod error;
mod handle;
mod resolve;
mod verify;

pub use api::*;
pub use cache::*;
pub use error::*;
pub use handle::*;
pub use resolve::*;
pub use verify::*;

#[cfg(test)]
mod tests;
