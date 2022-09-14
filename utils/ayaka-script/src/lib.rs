//! The script and text parsers.

#![warn(missing_docs)]
#![deny(unsafe_code)]
#![feature(iterator_try_collect)]
#![feature(once_cell)]

mod exec;
mod text;

pub use exec::*;
#[doc(no_inline)]
pub use log;
pub use text::*;
