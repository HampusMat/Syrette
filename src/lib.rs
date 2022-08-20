#![cfg_attr(feature = "factory", feature(unboxed_closures, fn_traits))]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(missing_docs)]

//! Syrette
//!
//! Syrette is a collection of utilities useful for performing dependency injection.

pub mod di_container;
pub mod errors;
pub mod interfaces;
pub mod ptr;

pub use di_container::DIContainer;
pub use syrette_macros::*;

#[cfg(feature = "factory")]
#[doc(hidden)]
pub mod castable_factory;

#[doc(hidden)]
pub mod dependency_trace;

#[doc(hidden)]
pub mod libs;

// Private
mod di_container_binding_map;
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
        $di_container.bind::<dyn $interface>().to::<$implementation>().unwrap();

        syrette::declare_interface!($implementation -> $interface);
    };
}

/// Shortcut for declaring a default factory.
///
/// A default factory is a factory that doesn't take any arguments.
///
/// More tedious ways to accomplish what this macro does would either be by using
/// the [`factory`] macro or by manually declaring the interfaces
/// with the [`declare_interface`] macro.
///
/// *This macro is only available if Syrette is built with the "factory" feature.*
///
/// # Arguments
/// - Interface trait
///
/// # Examples
/// ```
/// use syrette::declare_default_factory;
///
/// trait IParser {
///     // Methods and etc here...
/// }
///
/// declare_default_factory!(IParser);
/// ```
///
/// The expanded equivelent of this would be
///
/// ```
/// use syrette::declare_default_factory;
///
/// trait IParser {
///     // Methods and etc here...
/// }
///
/// syrette::declare_interface!(
///     syrette::castable_factory::CastableFactory<
///         (),
///         dyn IParser,
///     > -> syrette::interfaces::factory::IFactory<(), dyn IParser>
/// );
///
/// syrette::declare_interface!(
///     syrette::castable_factory::CastableFactory<
///         (),
///         dyn IParser,
///     > -> syrette::interfaces::any_factory::AnyFactory
/// );
/// ```
#[macro_export]
#[cfg(feature = "factory")]
macro_rules! declare_default_factory {
    ($interface: path) => {
        syrette::declare_interface!(
            syrette::castable_factory::CastableFactory<
                (),
                dyn $interface,
            > -> syrette::interfaces::factory::IFactory<(), dyn $interface>
        );

        syrette::declare_interface!(
            syrette::castable_factory::CastableFactory<
                (),
                dyn $interface,
            > -> syrette::interfaces::any_factory::AnyFactory
        );
    }
}
