//! The types used in both runtime and plugins.

#![warn(missing_docs)]
#![deny(unsafe_code)]
#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]

mod logger;
pub use logger::*;

mod plugin;
pub use plugin::*;

mod config;
pub use config::*;

mod runtime;
pub use runtime::*;

mod script;
pub use script::*;
