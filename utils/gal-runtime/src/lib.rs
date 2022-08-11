//! The runtime of `gal` project.
//!
//! This runtime provides the game config, running context,
//! plugin system and settings system.
//! It can be treated as the "backend" of the game engine.

#![warn(missing_docs)]
#![deny(unsafe_code)]
#![feature(absolute_path)]
#![feature(async_closure)]
#![feature(once_cell)]
#![feature(round_char_boundary)]
#![feature(trait_alias)]

mod config;
mod context;
pub mod plugin;
mod progress_future;
pub mod script;
mod settings;

pub use anyhow;
pub use config::*;
pub use context::*;
pub use gal_locale::{Locale, LocaleBuf};
pub use gal_script::{log, RawValue};
pub use progress_future::*;
pub use settings::*;
pub use tokio;
pub use tokio_stream;
