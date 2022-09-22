//! The script and text parsers.

#![warn(missing_docs)]
#![deny(unsafe_code)]
#![feature(iterator_try_collect)]
#![feature(once_cell)]

mod exec;
mod line;
mod primitive;
mod text;

pub use exec::*;
pub use line::*;
#[doc(no_inline)]
pub use log;
pub use primitive::*;
pub use text::*;
