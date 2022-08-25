//! The runtime of `gal` project.
//!
//! This runtime provides the game config, running context,
//! plugin system and settings system.
//! It can be treated as the "backend" of the game engine.

#![warn(missing_docs)]
#![deny(unsafe_code)]
#![feature(absolute_path)]
#![feature(async_closure)]
#![feature(generators)]
#![feature(once_cell)]
#![feature(round_char_boundary)]
#![feature(trait_alias)]

mod config;
mod context;
pub mod plugin;
pub mod script;
mod settings;

pub use anyhow;
pub use config::*;
pub use context::*;
pub use gal_script::{log, RawValue};
pub use locale::*;
pub use settings::*;
pub use stream_future::*;
pub use tokio;
pub use tokio_stream;
