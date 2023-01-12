#![cfg_attr(doc_cfg, feature(doc_cfg))]
#![deny(clippy::all, clippy::pedantic, missing_docs, unsafe_code)]
#![allow(unknown_lints)]
#![allow(clippy::module_name_repetitions, clippy::manual_let_else)]

//! Macros for the [Syrette](https://crates.io/crates/syrette) crate.

use proc_macro::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::token::Dyn;
use syn::{
    parse,
    parse_macro_input,
    TraitBound,
    TraitBoundModifier,
    Type,
    TypeParamBound,
    TypeTraitObject,
};

mod caster;
mod declare_interface_args;
mod injectable;
mod macro_flag;
mod util;

#[cfg(feature = "factory")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "factory")))]
mod factory;

#[cfg(feature = "factory")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "factory")))]
mod fn_trait;

#[cfg(test)]
mod test_utils;

use crate::caster::generate_caster;
use crate::declare_interface_args::DeclareInterfaceArgs;
use crate::injectable::dependency::Dependency;
use crate::injectable::implementation::InjectableImpl;
use crate::injectable::macro_args::InjectableMacroArgs;

/// Makes a struct injectable. Thereby usable with [`DIContainer`] or
/// [`AsyncDIContainer`].
///
/// # Arguments
/// * (Optional) A interface trait the struct implements.
/// * (Zero or more) Flags. Like `a = true, b = false`
///
/// # Flags
/// - `no_doc_hidden` - Don't hide the impl of the [`Injectable`] trait from
///   documentation.
/// - `no_declare_concrete_interface` - Disable declaring the concrete type as the
///   interface when no interface trait argument is given.
/// - `async` - Mark as async.
///
/// # Panics
/// If the attributed item is not a impl.
///
/// # Important
/// When no interface trait argument is given, you should either manually
/// declare the interface with the [`declare_interface!`] macro or use
/// the [`di_container_bind`] macro to create a DI container binding.
///
/// You can however also use the concrete type as the interface. As it is declared as such
/// by default if the `no_declare_concrete_interface` flag is not set.
///
/// # Attributes
/// Attributes specific to impls with this attribute macro.
///
/// ### Named
/// Used inside the `new` method before a dependency argument. Declares the name of the
/// dependency. Should be given the name quoted inside parenthesis.
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
/// # Example
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
/// [`DIContainer`]: https://docs.rs/syrette/latest/syrette/di_container/struct.DIContainer.html
/// [`AsyncDIContainer`]: https://docs.rs/syrette/latest/syrette/async_di_container/struct.AsyncDIContainer.html
/// [`Injectable`]: https://docs.rs/syrette/latest/syrette/interfaces/injectable/trait.Injectable.html
/// [`di_container_bind`]: https://docs.rs/syrette/latest/syrette/macro.di_container_bind.html
#[cfg(not(tarpaulin_include))]
#[proc_macro_attribute]
pub fn injectable(args_stream: TokenStream, impl_stream: TokenStream) -> TokenStream
{
    let InjectableMacroArgs { interface, flags } = parse_macro_input!(args_stream);

    let no_doc_hidden = flags
        .iter()
        .find(|flag| flag.flag.to_string().as_str() == "no_doc_hidden")
        .map_or(false, |flag| flag.is_on.value);

    let no_declare_concrete_interface = flags
        .iter()
        .find(|flag| flag.flag.to_string().as_str() == "no_declare_concrete_interface")
        .map_or(false, |flag| flag.is_on.value);

    let is_async = flags
        .iter()
        .find(|flag| flag.flag.to_string().as_str() == "async")
        .map_or(false, |flag| flag.is_on.value);

    let injectable_impl: InjectableImpl<Dependency> = match parse(impl_stream) {
        Ok(injectable_impl) => injectable_impl,
        Err(err) => {
            panic!("{err}");
        }
    };

    let expanded_injectable_impl = injectable_impl.expand(no_doc_hidden, is_async);

    let self_type = &injectable_impl.self_type;

    let opt_interface = interface.map(Type::Path).or_else(|| {
        if no_declare_concrete_interface {
            None
        } else {
            Some(self_type.clone())
        }
    });

    let maybe_decl_interface = if let Some(interface) = opt_interface {
        let async_flag = if is_async {
            quote! {, async = true}
        } else {
            quote! {}
        };

        quote! {
            syrette::declare_interface!(#self_type -> #interface #async_flag);
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
/// The return type is automatically put inside of a [`TransientPtr`].
///
/// # Arguments
/// * (Zero or more) Flags. Like `a = true, b = false`
///
/// # Flags
/// - `threadsafe` - Mark as threadsafe.
/// - `async` - Mark as async. Infers the `threadsafe` flag. The return type is
///   automatically put inside of a pinned boxed future.
///
/// # Panics
/// If the attributed item is not a type alias.
///
/// # Examples
/// ```
/// # use syrette::factory;
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
/// type IConfiguratorFactory = dyn Fn(Vec<String>) -> dyn IConfigurator;
/// ```
///
/// [`TransientPtr`]: https://docs.rs/syrette/latest/syrette/ptr/type.TransientPtr.html
#[cfg(feature = "factory")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "factory")))]
#[cfg(not(tarpaulin_include))]
#[proc_macro_attribute]
pub fn factory(args_stream: TokenStream, type_alias_stream: TokenStream) -> TokenStream
{
    use quote::ToTokens;
    use syn::parse_str;

    use crate::factory::build_declare_interfaces::build_declare_factory_interfaces;
    use crate::factory::macro_args::FactoryMacroArgs;
    use crate::factory::type_alias::FactoryTypeAlias;

    let FactoryMacroArgs { flags } = parse(args_stream).unwrap();

    let mut is_threadsafe = flags
        .iter()
        .find(|flag| flag.flag.to_string().as_str() == "threadsafe")
        .map_or(false, |flag| flag.is_on.value);

    let is_async = flags
        .iter()
        .find(|flag| flag.flag.to_string().as_str() == "async")
        .map_or(false, |flag| flag.is_on.value);

    if is_async {
        is_threadsafe = true;
    }

    let FactoryTypeAlias {
        mut type_alias,
        mut factory_interface,
        arg_types: _,
        return_type: _,
    } = parse(type_alias_stream).unwrap();

    let output = factory_interface.output.clone();

    factory_interface.output = parse(
        if is_async {
            quote! {
                syrette::future::BoxFuture<'static, syrette::ptr::TransientPtr<#output>>
            }
        } else {
            quote! {
                syrette::ptr::TransientPtr<#output>
            }
        }
        .into(),
    )
    .unwrap();

    if is_threadsafe {
        factory_interface.add_trait_bound(parse_str("Send").unwrap());
        factory_interface.add_trait_bound(parse_str("Sync").unwrap());
    }

    type_alias.ty = Box::new(Type::Verbatim(factory_interface.to_token_stream()));

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
/// # Panics
/// If the provided arguments are invalid.
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
#[proc_macro]
pub fn declare_default_factory(args_stream: TokenStream) -> TokenStream
{
    use syn::parse_str;

    use crate::factory::build_declare_interfaces::build_declare_factory_interfaces;
    use crate::factory::declare_default_args::DeclareDefaultFactoryMacroArgs;
    use crate::fn_trait::FnTrait;

    let DeclareDefaultFactoryMacroArgs { interface, flags } = parse(args_stream).unwrap();

    let mut is_threadsafe = flags
        .iter()
        .find(|flag| flag.flag.to_string().as_str() == "threadsafe")
        .map_or(false, |flag| flag.is_on.value);

    let is_async = flags
        .iter()
        .find(|flag| flag.flag.to_string().as_str() == "async")
        .map_or(false, |flag| flag.is_on.value);

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
    .unwrap();

    if is_threadsafe {
        factory_interface.add_trait_bound(parse_str("Send").unwrap());
        factory_interface.add_trait_bound(parse_str("Sync").unwrap());
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
/// - `async` - Mark as async.
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
#[proc_macro]
pub fn declare_interface(input: TokenStream) -> TokenStream
{
    let DeclareInterfaceArgs {
        implementation,
        interface,
        flags,
    } = parse_macro_input!(input);

    let opt_async_flag = flags
        .iter()
        .find(|flag| flag.flag.to_string().as_str() == "async");

    let is_async =
        opt_async_flag.map_or_else(|| false, |async_flag| async_flag.is_on.value);

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
