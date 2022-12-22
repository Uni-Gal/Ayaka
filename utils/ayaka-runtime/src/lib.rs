//! The runtime of Ayaka project.
//!
//! This runtime provides the game config, running context,
//! plugin system and settings system.
//! It can be treated as the "backend" of the game engine.

#![warn(missing_docs)]
#![deny(unsafe_code)]
#![feature(generators)]
#![feature(once_cell)]

mod config;
mod context;
mod locale;
pub mod plugin;
pub mod settings;
mod view_model;

#[doc(no_inline)]
pub use anyhow;
#[doc(no_inline)]
pub use ayaka_bindings_types::*;
#[doc(no_inline)]
pub use ayaka_primitive::*;
pub use config::*;
pub use context::*;
#[doc(no_inline)]
pub use futures_util::{pin_mut, StreamExt, TryStreamExt};
#[doc(no_inline)]
pub use locale::*;
#[doc(no_inline)]
pub use log;
#[doc(no_inline)]
pub use vfs;
pub use view_model::*;

/// Get the version of Ayaka runtime.
/// This version string is exacted from `CARGO_PKG_VERSION`.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
