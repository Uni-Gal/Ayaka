//! The script and text parsers.

#![warn(missing_docs)]
#![deny(unsafe_code)]
#![feature(iterator_try_collect)]
#![feature(once_cell)]
#![feature(try_trait_v2)]

mod exec;
mod line;
mod primitive;
mod text;
mod tryext;

pub use exec::*;
pub use line::*;
#[doc(no_inline)]
pub use log;
pub use primitive::*;
pub use text::*;
pub use tryext::*;
