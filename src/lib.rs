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

pub use di_container::*;
pub use syrette_macros::*;

#[cfg(feature = "factory")]
#[doc(hidden)]
pub mod castable_factory;

#[doc(hidden)]
pub mod libs;

// Private
mod provider;

/// Shortcut for creating a DI container binding for a injectable without a declared interface.
///
/// This will declare a interface for the implementation.
///
/// Useful for when the implementation or the interface is generic.
///
/// # Arguments
/// {interface} => {implementation}, {DI container variable name}
#[macro_export]
macro_rules! di_container_bind {
    ($interface: path => $implementation: ty, $di_container: ident) => {
        $di_container.bind::<dyn $interface>().to::<$implementation>();

        syrette::declare_interface!($implementation -> $interface);
    };
}
