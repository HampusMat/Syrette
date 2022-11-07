#![cfg_attr(feature = "factory", feature(unboxed_closures, fn_traits, tuple_trait))]
#![cfg_attr(doc_cfg, feature(doc_cfg))]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![deny(missing_docs)]

//! Syrette
//!
//! Syrette is a collection of utilities useful for performing dependency injection.

pub mod dependency_history;
pub mod di_container;
pub mod errors;
pub mod interfaces;
pub mod ptr;

#[cfg(feature = "async")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "async")))]
pub mod future;

#[cfg(feature = "async")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "async")))]
pub use di_container::asynchronous::AsyncDIContainer;
pub use di_container::blocking::DIContainer;
#[cfg(feature = "factory")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "factory")))]
pub use syrette_macros::{declare_default_factory, factory};
pub use syrette_macros::{declare_interface, injectable, named};

#[cfg(feature = "factory")]
#[doc(hidden)]
pub mod castable_factory;

#[doc(hidden)]
pub mod libs;

// Private
mod provider;

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod test_utils;

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
/// # use syrette::di_container::blocking::IDIContainer;
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
#[cfg(not(tarpaulin_include))]
#[macro_export]
macro_rules! di_container_bind {
    ($interface: path => $implementation: ty, $di_container: ident) => {
        $di_container.bind::<dyn $interface>().to::<$implementation>().unwrap();

        syrette::declare_interface!($implementation -> $interface);
    };
}

/// Creates a async closure.
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
#[cfg_attr(doc_cfg, doc(cfg(feature = "async")))]
#[cfg(not(tarpaulin_include))]
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
