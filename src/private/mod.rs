//! This module contains items that's not in the public API but is used by the
//! library user with the expansions of the macros in the syrette_macros crate.

pub mod cast;

pub extern crate linkme;

#[cfg(feature = "factory")]
pub mod any_factory;

#[cfg(feature = "factory")]
pub mod factory;

#[cfg(feature = "factory")]
pub mod castable_factory;
