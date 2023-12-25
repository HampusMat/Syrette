#![cfg_attr(doc_cfg, feature(doc_cfg))]
#![deny(clippy::all, clippy::pedantic, missing_docs, unsafe_code)]
#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::panic))]
#![allow(unknown_lints)]
#![allow(clippy::module_name_repetitions, clippy::manual_let_else)]

//! Macros for the [Syrette](https://crates.io/crates/syrette) crate.

use proc_macro::TokenStream;
use proc_macro_error::{proc_macro_error, set_dummy, ResultExt};
use quote::{format_ident, quote};
use syn::{parse, ItemImpl};

use crate::injectable::dummy::expand_dummy_blocking_impl;
use crate::injectable::implementation::{InjectableImpl, InjectableImplError};
use crate::injectable::macro_args::InjectableMacroArgs;
use crate::macro_flag::MacroFlag;

mod injectable;
mod macro_flag;
mod util;

#[cfg(test)]
mod test_utils;

#[cfg(feature = "async")]
use crate::injectable::dummy::expand_dummy_async_impl;

#[allow(dead_code)]
const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Makes a type injectable.
///
/// Generates an implementation of [`Injectable`].
///
/// # Arguments
/// * (Optional) A interface trait the struct implements.
/// * (Zero or more) Comma separated flags. Each flag being formatted `name=value`.
///
/// # Flags
/// #### `no_doc_hidden`
/// **Value:** boolean literal<br>
/// **Default:** `false`<br>
/// Don't hide the impl of the [`Injectable`] trait from documentation.
///
/// #### `async`
/// <span class="cf">Available on <strong>crate feature <code>async</code></strong> only.
/// </span>
///
/// **Value:** boolean literal<br>
/// **Default:** `false`<br>
/// Generate an implementation of [`AsyncInjectable`] instead of [`Injectable`].
///
/// This flag must be set to `true` for the type to be usable with [`AsyncDIContainer`].
///
/// #### `constructor`
/// **Value:** identifier<br>
/// **Default:** `new`<br>
/// Constructor method name.
///
/// # Important
/// When no interface trait argument is given, the concrete type is used as a interface.
///
/// # Examples
/// ```
/// # use syrette::injectable;
/// #
/// # struct PasswordManager {}
/// #
/// #[injectable]
/// impl PasswordManager
/// {
///     pub fn new() -> Self
///     {
///         Self {}
///     }
/// }
/// ```
/// <br>
///
/// When the `async` crate feature is enabled, the constructor can be [`async`].
/// ```
/// # #[cfg(feature = "async")]
/// # mod example {
/// # use syrette::injectable;
/// #
/// # struct PasswordManager { value: u32}
/// #
/// # async fn some_async_computation() -> u32 { 1 }
/// #
/// #[injectable(async = true)]
/// impl PasswordManager
/// {
///     pub async fn new() -> Self
///     {
///         let value = some_async_computation().await;
///
///         Self { value }
///     }
/// }
/// # }
/// ```
///
/// # Attributes
/// Attributes specific to impls with this attribute macro.
///
/// ### Named
/// Used inside the of constructor method before a dependency argument. Declares the name
/// of the dependency. Should be given the name quoted inside parenthesis.
///
/// The [`macro@named`] ghost attribute macro can be used for intellisense and
/// autocompletion for this attribute.
///
/// For example:
/// ```
/// # use syrette::ptr::TransientPtr;
/// # use syrette::injectable;
/// #
/// # trait IArmor {}
/// #
/// # trait IKnight {}
/// #
/// # struct Knight
/// # {
/// #     tough_armor: TransientPtr<dyn IArmor>,
/// #     light_armor: TransientPtr<dyn IArmor>,
/// # }
/// #
/// #[injectable(IKnight)]
/// impl Knight
/// {
///     pub fn new(
///         #[named("tough")] tough_armor: TransientPtr<dyn IArmor>,
///         #[named("light")] light_armor: TransientPtr<dyn IArmor>,
///     ) -> Self
///     {
///         Self {
///             tough_armor,
///             light_armor,
///         }
///     }
/// }
/// #
/// # impl IKnight for Knight {}
/// ```
///
/// [`DIContainer`]: ../syrette/di_container/blocking/struct.DIContainer.html
/// [`AsyncDIContainer`]: ../syrette/di_container/asynchronous/struct.AsyncDIContainer.html
/// [`Injectable`]: ../syrette/interfaces/injectable/trait.Injectable.html
/// [`AsyncInjectable`]: ../syrette/interfaces/async_injectable/trait.AsyncInjectable.html
/// [`di_container_bind`]: ../syrette/macro.di_container_bind.html
/// [`async`]: https://doc.rust-lang.org/std/keyword.async.html
#[cfg(not(tarpaulin_include))]
#[proc_macro_error]
#[proc_macro_attribute]
pub fn injectable(args_stream: TokenStream, input_stream: TokenStream) -> TokenStream
{
    let item_impl = parse::<ItemImpl>(input_stream)
        .map_err(|err| InjectableImplError::NotAImplementation {
            err_span: err.span(),
        })
        .unwrap_or_abort();

    let dummy_blocking_impl =
        expand_dummy_blocking_impl(&item_impl.generics, &item_impl.self_ty);

    #[cfg(not(feature = "async"))]
    set_dummy(quote! {
        #item_impl
        #dummy_blocking_impl
    });

    #[cfg(feature = "async")]
    {
        let dummy_async_impl =
            expand_dummy_async_impl(&item_impl.generics, &item_impl.self_ty);

        set_dummy(quote! {
            #item_impl
            #dummy_blocking_impl
            #dummy_async_impl
        });
    }

    let args = parse::<InjectableMacroArgs>(args_stream).unwrap_or_abort();

    args.check_flags().unwrap_or_abort();

    let no_doc_hidden = args
        .flags
        .iter()
        .find(|flag| flag.name() == "no_doc_hidden")
        .map_or(Ok(false), MacroFlag::get_bool)
        .unwrap_or_abort();

    let constructor = args
        .flags
        .iter()
        .find(|flag| flag.name() == "constructor")
        .map_or(Ok(format_ident!("new")), MacroFlag::get_ident)
        .unwrap_or_abort();

    let is_async_flag = args
        .flags
        .iter()
        .find(|flag| flag.name() == "async")
        .cloned()
        .unwrap_or_else(|| MacroFlag::new_off("async"));

    let is_async = is_async_flag.get_bool().unwrap_or_abort();

    #[cfg(not(feature = "async"))]
    if is_async {
        use proc_macro_error::abort;

        abort!(
            is_async_flag.name().span(),
            "The 'async' crate feature must be enabled to use this flag";
            suggestion = concat!(
                "In your Cargo.toml: syrette = {{ version = \"{}\", features = ",
                "[\"async\"] }}"
            ),
            PACKAGE_VERSION
        );
    }

    let injectable_impl = InjectableImpl::new(item_impl, &constructor).unwrap_or_abort();

    injectable_impl.validate(is_async).unwrap_or_abort();

    let expanded_injectable_impl =
        injectable_impl.expand(no_doc_hidden, is_async, args.interface.as_ref());

    quote! {
        #expanded_injectable_impl
    }
    .into()
}

/// Used to declare the name of a dependency in the constructor of a impl block decorated
/// with [`macro@injectable`].
///
/// **This macro attribute doesn't actually do anything**. It only exists for the
/// convenience of having intellisense, autocompletion and documentation.
///
/// # Examples
/// ```
/// # use syrette::ptr::TransientPtr;
/// # use syrette::injectable;
/// #
/// # trait INinja {}
/// # trait IWeapon {}
/// #
/// # struct Ninja
/// # {
/// #   strong_weapon: TransientPtr<dyn IWeapon>,
/// #   weak_weapon: TransientPtr<dyn IWeapon>,
/// # }
/// #
/// #[injectable(INinja)]
/// impl Ninja
/// {
///     pub fn new(
///         #[syrette::named("strong")] strong_weapon: TransientPtr<dyn IWeapon>,
///         #[syrette::named("weak")] weak_weapon: TransientPtr<dyn IWeapon>,
///     ) -> Self
///     {
///         Self {
///             strong_weapon,
///             weak_weapon,
///         }
///     }
/// }
/// #
/// # impl INinja for Ninja {}
/// ```
#[cfg(not(tarpaulin_include))]
#[proc_macro_attribute]
pub fn named(_: TokenStream, _: TokenStream) -> TokenStream
{
    TokenStream::new()
}

#[cfg(syrette_macros_logging)]
fn init_logging()
{
    use tracing::Level;
    use tracing_subscriber::FmtSubscriber;

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();

    // The error can be ignored because it doesn't matter if the global default
    // has already been set
    tracing::subscriber::set_global_default(subscriber).ok();
}
