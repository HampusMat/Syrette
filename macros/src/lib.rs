#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(missing_docs)]

//! Macros for the [Syrette](https://crates.io/crates/syrette) crate.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse, parse_macro_input};

mod declare_interface_args;
mod dependency;
mod factory_type_alias;
mod injectable_impl;
mod injectable_macro_args;
mod libs;
mod named_attr_input;
mod util;

use declare_interface_args::DeclareInterfaceArgs;
use injectable_impl::InjectableImpl;
use injectable_macro_args::InjectableMacroArgs;
use libs::intertrait_macros::gen_caster::generate_caster;

/// Makes a struct injectable. Thereby usable with [`DIContainer`].
///
/// # Arguments
/// * (Optional) A interface trait the struct implements.
/// * (Zero or more) Flags wrapped in curly braces. Like `{ a = true, b = false }`
///
/// # Flags
/// - `no_doc_hidden` - Don't hide the impl of the [`Injectable`] trait from documentation.
///
/// # Panics
/// If the attributed item is not a impl.
///
/// # Important
/// If the interface trait argument is excluded, you should either manually
/// declare the interface with the [`declare_interface!`] macro or use
/// the [`di_container_bind`] macro to create a DI container binding.
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
///         #[named("tough")]
///         tough_armor: TransientPtr<dyn IArmor>,
///
///         #[named("light")]
///         light_armor: TransientPtr<dyn IArmor>
///     ) -> Self
///     {
///         Self { tough_armor, light_armor }
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
/// [`DIContainer`]: ../syrette/di_container/struct.DIContainer.html
/// [`Injectable`]: ../syrette/interfaces/injectable/trait.Injectable.html
/// [`di_container_bind`]: ../syrette/macro.di_container_bind.html
#[proc_macro_attribute]
pub fn injectable(args_stream: TokenStream, impl_stream: TokenStream) -> TokenStream
{
    let InjectableMacroArgs { interface, flags } = parse_macro_input!(args_stream);

    let mut flags_iter = flags.iter();

    let no_doc_hidden = flags_iter
        .find(|flag| flag.flag.to_string().as_str() == "no_doc_hidden")
        .map_or(false, |flag| flag.is_on.value);

    let injectable_impl: InjectableImpl = parse(impl_stream).unwrap();

    let expanded_injectable_impl = injectable_impl.expand(no_doc_hidden);

    let maybe_decl_interface = if interface.is_some() {
        let self_type = &injectable_impl.self_type;

        quote! {
            syrette::declare_interface!(#self_type -> #interface);
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
/// *This macro is only available if Syrette is built with the "factory" feature.*
///
/// # Panics
/// If the attributed item is not a type alias.
///
/// # Examples
/// ```
/// use std::collections::HashMap;
///
/// use syrette::interfaces::factory::IFactory;
/// use syrette::factory;
///
/// enum ConfigValue
/// {
///     String(String),
///     Bool(bool),
///     Int(i32),
///     None,
/// }
///
/// trait IConfigurator
/// {
///     fn configure(&self, key: String, value: ConfigValue);
/// }
///
/// struct Configurator {
///     config: HashMap<String, ConfigValue>,
/// }
///
/// impl Configurator
/// {
///     fn new(keys: Vec<String>) -> Self
///     {
///         Self {
///             config: HashMap::from(
///                 keys
///                     .iter()
///                     .map(|key| (key.clone(), ConfigValue::None))
///                     .collect::<HashMap<_, _>>()
///             )
///         }
///     }
/// }
///
/// impl IConfigurator for Configurator
/// {
///     fn configure(&self, key: String, value: ConfigValue)
///     {
///         // ...
///     }
/// }
///
/// #[factory]
/// type IConfiguratorFactory = dyn IFactory<(Vec<String>,), dyn IConfigurator>;
/// ```
#[proc_macro_attribute]
#[cfg(feature = "factory")]
pub fn factory(_: TokenStream, type_alias_stream: TokenStream) -> TokenStream
{
    let factory_type_alias::FactoryTypeAlias {
        type_alias,
        factory_interface,
        arg_types,
        return_type,
    } = parse(type_alias_stream).unwrap();

    quote! {
        #type_alias

        syrette::declare_interface!(
            syrette::castable_factory::CastableFactory<
                #arg_types,
                #return_type
            > -> #factory_interface
        );

        syrette::declare_interface!(
            syrette::castable_factory::CastableFactory<
                #arg_types,
                #return_type
            > -> syrette::interfaces::any_factory::AnyFactory
        );
    }
    .into()
}

/// Declares the interface trait of a implementation.
///
/// # Arguments
/// {Implementation} -> {Interface}
///
#[proc_macro]
pub fn declare_interface(input: TokenStream) -> TokenStream
{
    let DeclareInterfaceArgs {
        implementation,
        interface,
    } = parse_macro_input!(input);

    generate_caster(&implementation, &interface).into()
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
///         #[syrette::named("strong")]
///         strong_weapon: TransientPtr<dyn IWeapon>,
///
///         #[syrette::named("weak")]
///         weak_weapon: TransientPtr<dyn IWeapon>,
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
#[proc_macro_attribute]
pub fn named(_: TokenStream, _: TokenStream) -> TokenStream
{
    TokenStream::new()
}
