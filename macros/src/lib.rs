#![cfg_attr(doc_cfg, feature(doc_cfg))]
#![deny(clippy::all, clippy::pedantic, missing_docs, unsafe_code)]
#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::panic))]
#![allow(unknown_lints)]
#![allow(clippy::module_name_repetitions, clippy::manual_let_else)]

//! Macros for the [Syrette](https://crates.io/crates/syrette) crate.

use proc_macro::TokenStream;
use proc_macro_error::{proc_macro_error, set_dummy, ResultExt};
use quote::{format_ident, quote};
use syn::punctuated::Punctuated;
use syn::token::Dyn;
use syn::{
    parse,
    ItemImpl,
    TraitBound,
    TraitBoundModifier,
    Type,
    TypeParamBound,
    TypeTraitObject,
};

use crate::caster::generate_caster;
use crate::declare_interface_args::DeclareInterfaceArgs;
use crate::injectable::dependency::Dependency;
use crate::injectable::dummy::expand_dummy_blocking_impl;
use crate::injectable::implementation::{InjectableImpl, InjectableImplError};
use crate::injectable::macro_args::InjectableMacroArgs;
use crate::macro_flag::MacroFlag;

mod caster;
mod declare_interface_args;
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
/// #### `no_declare_concrete_interface`
/// **Value:** boolean literal<br>
/// **Default:** `false`<br>
/// Disable declaring the concrete type as the interface when no interface trait argument
/// is given.
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
/// When no interface trait argument is given, you have three options
/// - Manually declare the interface with the [`declare_interface!`] macro.
/// - Use the [`di_container_bind`] macro to create a DI container binding.
/// - Use the concrete type as the interface.
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

    let no_declare_concrete_interface = args
        .flags
        .iter()
        .find(|flag| flag.name() == "no_declare_concrete_interface")
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

    let injectable_impl =
        InjectableImpl::<Dependency>::new(item_impl, &constructor).unwrap_or_abort();

    injectable_impl.validate().unwrap_or_abort();

    let expanded_injectable_impl = injectable_impl.expand(no_doc_hidden, is_async);

    let self_type = injectable_impl.self_type();

    let opt_interface = args.interface.map(Type::Path).or_else(|| {
        if no_declare_concrete_interface {
            None
        } else {
            Some(self_type.clone())
        }
    });

    let maybe_decl_interface = if let Some(interface) = opt_interface {
        let threadsafe_sharable_flag = if is_async {
            quote! { , threadsafe_sharable = true }
        } else {
            quote! {}
        };

        quote! {
            syrette::declare_interface!(
                #self_type -> #interface #threadsafe_sharable_flag
            );
        }
    } else {
        quote! {}
    };

    quote! {
        #expanded_injectable_impl

        #maybe_decl_interface
    }
    .into()
}

/// Declares the interface trait of a implementation.
///
/// # Arguments
/// {Implementation} -> {Interface}
/// * (Zero or more) Flags. Like `a = true, b = false`
///
/// # Flags
/// - `threadsafe_sharable` - Enables the use of thread-safe shared instances of the
///   implementation accessed with the interface.
///
/// # Examples
/// ```
/// # use syrette::declare_interface;
/// #
/// # trait INinja {}
/// #
/// # struct Ninja {}
/// #
/// # impl INinja for Ninja {}
/// #
/// declare_interface!(Ninja -> INinja);
/// ```
#[cfg(not(tarpaulin_include))]
#[proc_macro_error]
#[proc_macro]
pub fn declare_interface(input: TokenStream) -> TokenStream
{
    let DeclareInterfaceArgs {
        implementation,
        interface,
        flags,
    } = parse(input).unwrap_or_abort();

    let threadsafe_sharable_flag = flags
        .iter()
        .find(|flag| flag.name() == "threadsafe_sharable");

    let is_async = threadsafe_sharable_flag
        .map_or_else(|| Ok(false), MacroFlag::get_bool)
        .unwrap_or_abort();

    #[cfg(syrette_macros_logging)]
    init_logging();

    let interface_type = if interface == implementation {
        Type::Path(interface)
    } else {
        Type::TraitObject(TypeTraitObject {
            dyn_token: Some(Dyn::default()),
            bounds: Punctuated::from_iter(vec![TypeParamBound::Trait(TraitBound {
                paren_token: None,
                modifier: TraitBoundModifier::None,
                lifetimes: None,
                path: interface.path,
            })]),
        })
    };

    generate_caster(&implementation, &interface_type, is_async).into()
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
