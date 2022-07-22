#![cfg_attr(feature = "factory", feature(unboxed_closures, fn_traits))]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]

//! Syrette
//!
//! Syrette is a collection of utilities useful for performing dependency injection.

pub mod di_container;
pub mod errors;
pub mod interfaces;
pub mod ptr;

#[cfg(feature = "factory")]
pub mod castable_factory;

pub use di_container::*;
pub use syrette_macros::*;

#[doc(hidden)]
pub mod libs;

// Private
mod provider;
