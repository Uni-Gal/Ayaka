#![feature(absolute_path)]
#![feature(async_closure)]
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
pub use settings::*;
pub use tokio;
pub use tokio_stream;
