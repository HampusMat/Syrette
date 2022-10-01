#![cfg_attr(feature = "factory", feature(unboxed_closures, fn_traits))]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
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
pub mod future;

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

/// Creates a async closure.
///
/// *This macro is only available if Syrette is built with the "async" feature.*
///
/// # Examples
/// ```
/// # use syrette::async_closure;
/// #
/// # async fn do_heavy_operation(timeout: u32, size: u32) -> String { String::new() }
/// #
/// # async fn do_other_heavy_operation(input: String) -> String { String::new() }
/// #
/// async_closure!(|timeout, size| {
///     let value = do_heavy_operation(timeout, size).await;
///
///     let final_value = do_other_heavy_operation(value).await;
///
///     final_value
/// });
/// ```
///
/// expands to the following
///
/// ```rust
/// # async fn do_heavy_operation(timeout: u32, size: u32) -> String { String::new() }
/// #
/// # async fn do_other_heavy_operation(input: String) -> String { String::new() }
/// #
/// Box::new(|timeout, size| {
///     Box::pin(async move {
///         let value = do_heavy_operation(timeout, size).await;
///
///         let final_value = do_other_heavy_operation(value).await;
///
///         final_value
///     })
/// });
/// ```
#[cfg(feature = "async")]
#[macro_export]
macro_rules! async_closure {
    (|$($args: ident),*| { $($inner: stmt);* }) => {
        Box::new(|$($args),*| {
            Box::pin(async move { $($inner)* })
        })
    };
    (|| { $($inner: stmt);* }) => {
        Box::new(|| {
            Box::pin(async move { $($inner)* })
        })
    };
}
