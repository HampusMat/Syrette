#![feature(unboxed_closures, fn_traits)]

//! Syrette
//!
//! Syrette is a collection of utilities useful for performing dependency injection.

pub mod castable_factory;
pub mod di_container;
pub mod errors;
pub mod interfaces;
pub mod ptr;

pub use di_container::*;
pub use syrette_macros::*;

#[doc(hidden)]
pub mod libs;

// Private
mod provider;
