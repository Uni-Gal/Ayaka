//! Primitive types.

#![warn(missing_docs)]
#![deny(unsafe_code)]
#![feature(iterator_try_collect)]
#![feature(once_cell)]

mod line;
mod raw_value;
mod text;

pub use line::*;
pub use raw_value::*;
pub use text::*;
