//! The high level wrapper model of Ayaka.
//!
//! This crate provides a view model for a full-functionality frontend,
//! and a abstract trait of settings manager.
//!
//! It re-exports the types of [`ayaka_runtime`].

#![warn(missing_docs)]
#![deny(unsafe_code)]
#![feature(generators)]
#![feature(return_position_impl_trait_in_trait)]
#![allow(incomplete_features)]

mod settings;
pub use settings::*;

mod view_model;
pub use view_model::*;

#[doc(no_inline)]
pub use ayaka_runtime::*;
