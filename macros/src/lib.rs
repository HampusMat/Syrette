#![cfg_attr(doc_cfg, feature(doc_cfg))]
#![deny(clippy::all, clippy::pedantic, missing_docs, unsafe_code)]
#![allow(unknown_lints)]
#![allow(clippy::module_name_repetitions, clippy::manual_let_else)]

//! Macros for the [Syrette](https://crates.io/crates/syrette) crate.

use proc_macro::TokenStream;
use proc_macro_error::{proc_macro_error, set_dummy, ResultExt};
use quote::quote;
use syn::punctuated::Punctuated;
use syn::token::Dyn;
use syn::{parse, TraitBound, TraitBoundModifier, Type, TypeParamBound, TypeTraitObject};

use crate::caster::generate_caster;
use crate::declare_interface_args::DeclareInterfaceArgs;
use crate::injectable::dependency::Dependency;
use crate::injectable::implementation::InjectableImpl;
use crate::injectable::macro_args::InjectableMacroArgs;
use crate::macro_flag::MacroFlag;

mod caster;
mod declare_interface_args;
mod injectable;
mod macro_flag;
mod util;

#[cfg(feature = "factory")]
mod factory;

#[cfg(feature = "factory")]
mod fn_trait;

#[cfg(test)]
mod test_utils;

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
    use quote::format_ident;

    let input_stream: proc_macro2::TokenStream = input_stream.into();

    set_dummy(input_stream.clone());

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
            suggestion = "In your Cargo.toml: syrette = {{ version = \"{}\", features = [\"async\"] }}",
            PACKAGE_VERSION
        );
    }

    let injectable_impl =
        InjectableImpl::<Dependency>::parse(input_stream, &constructor).unwrap_or_abort();

    set_dummy(if is_async {
        injectable_impl.expand_dummy_async_impl()
    } else {
        injectable_impl.expand_dummy_blocking_impl()
    });

    injectable_impl.validate().unwrap_or_abort();

    let expanded_injectable_impl = injectable_impl.expand(no_doc_hidden, is_async);

    let self_type = &injectable_impl.self_type;

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
            syrette::declare_interface!(#self_type -> #interface #threadsafe_sharable_flag);
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

/// Makes a type alias usable as a factory interface.
///
/// # Arguments
/// * (Zero or more) Flags. Like `a = true, b = false`
///
/// # Flags
/// - `threadsafe` - Mark as threadsafe.
///
/// # Examples
/// ```
/// # use syrette::factory;
/// # use syrette::ptr::TransientPtr;
/// #
/// # trait IConfigurator {}
/// #
/// # struct Configurator {}
/// #
/// # impl Configurator
/// # {
/// #     fn new() -> Self
/// #     {
/// #         Self {}
/// #     }
/// # }
/// #
/// # impl IConfigurator for Configurator {}
/// #
/// #[factory]
/// type IConfiguratorFactory = dyn Fn(Vec<String>) -> TransientPtr<dyn IConfigurator>;
/// ```
///
/// [`TransientPtr`]: https://docs.rs/syrette/latest/syrette/ptr/type.TransientPtr.html
#[cfg(feature = "factory")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "factory")))]
#[cfg(not(tarpaulin_include))]
#[proc_macro_error]
#[proc_macro_attribute]
pub fn factory(args_stream: TokenStream, input_stream: TokenStream) -> TokenStream
{
    use crate::factory::build_declare_interfaces::build_declare_factory_interfaces;
    use crate::factory::macro_args::FactoryMacroArgs;
    use crate::factory::type_alias::FactoryTypeAlias;

    set_dummy(input_stream.clone().into());

    let FactoryMacroArgs { flags } = parse(args_stream).unwrap_or_abort();

    let is_threadsafe = flags
        .iter()
        .find(|flag| flag.name() == "threadsafe")
        .map_or(Ok(false), MacroFlag::get_bool)
        .unwrap_or_abort();

    let FactoryTypeAlias {
        type_alias,
        factory_interface,
    } = parse(input_stream).unwrap_or_abort();

    let decl_interfaces =
        build_declare_factory_interfaces(&factory_interface, is_threadsafe);

    quote! {
        #type_alias

        #decl_interfaces
    }
    .into()
}

/// Shortcut for declaring a default factory.
///
/// A default factory is a factory that doesn't take any arguments.
///
/// Another way to accomplish what this macro does would be by using
/// the [`macro@factory`] macro.
///
/// # Arguments
/// - Interface trait
/// * (Zero or more) Flags. Like `a = true, b = false`
///
/// # Flags
/// - `threadsafe` - Mark as threadsafe.
/// - `async` - Mark as async. Infers the `threadsafe` flag.
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
#[cfg(feature = "factory")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "factory")))]
#[cfg(not(tarpaulin_include))]
#[proc_macro_error]
#[proc_macro]
pub fn declare_default_factory(args_stream: TokenStream) -> TokenStream
{
    use syn::parse_str;

    use crate::factory::build_declare_interfaces::build_declare_factory_interfaces;
    use crate::factory::declare_default_args::DeclareDefaultFactoryMacroArgs;
    use crate::fn_trait::FnTrait;

    let DeclareDefaultFactoryMacroArgs { interface, flags } =
        parse(args_stream).unwrap_or_abort();

    let mut is_threadsafe = flags
        .iter()
        .find(|flag| flag.name() == "threadsafe")
        .map_or(Ok(false), MacroFlag::get_bool)
        .unwrap_or_abort();

    let is_async = flags
        .iter()
        .find(|flag| flag.name() == "async")
        .map_or(Ok(false), MacroFlag::get_bool)
        .unwrap_or_abort();

    if is_async {
        is_threadsafe = true;
    }

    let mut factory_interface: FnTrait = parse(
        if is_async {
            quote! {
                dyn Fn() -> syrette::future::BoxFuture<
                    'static,
                    syrette::ptr::TransientPtr<#interface>
                >
            }
        } else {
            quote! {
                dyn Fn() -> syrette::ptr::TransientPtr<#interface>
            }
        }
        .into(),
    )
    .unwrap_or_abort();

    if is_threadsafe {
        factory_interface.add_trait_bound(parse_str("Send").unwrap_or_abort());
        factory_interface.add_trait_bound(parse_str("Sync").unwrap_or_abort());
    }

    build_declare_factory_interfaces(&factory_interface, is_threadsafe).into()
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

/// Declares the name of a dependency.
///
/// This macro attribute doesn't actually do anything. It only exists for the
/// convenience of having intellisense and autocompletion.
/// You might as well just use `named` if you don't care about that.
///
/// Only means something inside a `new` method inside a impl with
/// the [`macro@injectable`] macro attribute.
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
