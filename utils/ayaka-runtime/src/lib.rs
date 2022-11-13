//! The runtime of Ayaka project.
//!
//! This runtime provides the game config, running context,
//! plugin system and settings system.
//! It can be treated as the "backend" of the game engine.

#![warn(missing_docs)]
#![deny(unsafe_code)]
#![feature(absolute_path)]
#![feature(fn_traits)]
#![feature(generators)]
#![feature(once_cell)]
#![feature(tuple_trait)]
#![feature(unboxed_closures)]

mod config;
mod context;
mod locale;
pub mod plugin;
pub mod script;
mod settings;

#[doc(no_inline)]
pub use anyhow;
#[doc(no_inline)]
pub use ayaka_bindings_types::*;
#[doc(no_inline)]
pub use ayaka_script::log;
#[doc(no_inline)]
pub use ayaka_script::*;
pub use config::*;
pub use context::*;
#[doc(no_inline)]
pub use futures_util::{pin_mut, StreamExt, TryStreamExt};
#[doc(no_inline)]
pub use locale::*;
pub use settings::*;

/// Get the version of Ayaka runtime.
/// This version string is exacted from `CARGO_PKG_VERSION`.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
