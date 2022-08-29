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

#[cfg(feature = "async")]
pub mod async_di_container;

#[cfg(feature = "async")]
pub use async_di_container::AsyncDIContainer;
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

/// Shortcut for creating a DI container binding for a injectable without a declared
/// interface.
///
/// This will declare a interface for the implementation.
///
/// Useful for when the implementation or the interface is generic.
///
/// # Arguments
/// {interface} => {implementation}, {DI container variable name}
///
/// # Examples
/// ```
/// # use syrette::{di_container_bind, DIContainer, injectable};
/// #
/// # trait INinja {}
/// #
/// # struct Ninja {}
/// #
/// # #[injectable]
/// # impl Ninja
/// # {
/// #     fn new() -> Self
/// #     {
/// #         Self {}
/// #     }
/// # }
/// #
/// # impl INinja for Ninja {}
/// #
/// let mut di_container = DIContainer::new();
///
/// di_container_bind!(INinja => Ninja, di_container);
/// ```
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
/// The more tedious way to accomplish what this macro does would be by using
/// the [`factory`] macro.
///
/// *This macro is only available if Syrette is built with the "factory" feature.*
///
/// # Arguments
/// - Interface trait
///
/// # Examples
/// ```
/// # use syrette::declare_default_factory;
/// #
/// trait IParser
/// {
///     // Methods and etc here...
/// }
///
/// declare_default_factory!(dyn IParser);
/// ```
#[macro_export]
#[cfg(feature = "factory")]
macro_rules! declare_default_factory {
    ($interface: ty) => {
        syrette::declare_interface!(
            syrette::castable_factory::blocking::CastableFactory<
                (),
                $interface,
            > -> syrette::interfaces::factory::IFactory<(), $interface>
        );

        syrette::declare_interface!(
            syrette::castable_factory::blocking::CastableFactory<
                (),
                $interface,
            > -> syrette::interfaces::any_factory::AnyFactory
        );
    }
}
