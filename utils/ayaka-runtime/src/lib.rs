//! The runtime of Ayaka project.
//!
//! This runtime provides the game config, running context,
//! plugin system and settings system.
//! It can be treated as the "backend" of the game engine.

#![warn(missing_docs)]
#![deny(unsafe_code)]
#![feature(absolute_path)]
#![feature(generators)]
#![feature(once_cell)]
#![feature(round_char_boundary)]

mod config;
mod context;
mod locale;
pub mod plugin;
pub mod script;
mod settings;

#[doc(no_inline)]
pub use anyhow;
#[doc(no_inline)]
pub use ayaka_script::log;
#[doc(no_inline)]
pub use ayaka_script_types::RawValue;
pub use config::*;
pub use context::*;
#[doc(no_inline)]
pub use futures_util::{pin_mut, StreamExt, TryStreamExt};
#[doc(no_inline)]
pub use locale::*;
pub use settings::*;
