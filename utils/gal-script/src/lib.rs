//! The script and text parsers.

#![warn(missing_docs)]
#![feature(char_indices_offset)]
#![feature(iterator_try_collect)]

mod exec;
mod text;

pub use exec::*;
pub use gal_primitive::{RawValue, ValueType};
pub use log;
pub use text::*;
