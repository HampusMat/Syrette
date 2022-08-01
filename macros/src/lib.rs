#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(missing_docs)]

//! Macros for the [Syrette](https://crates.io/crates/syrette) crate.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse, parse_macro_input};

mod declare_interface_args;
mod dependency_type;
mod factory_type_alias;
mod injectable_impl;
mod injectable_macro_args;
mod libs;
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
/// # Example
/// ```
/// use syrette::injectable;
///
/// struct PasswordManager {}
///
/// #[injectable]
/// impl PasswordManager {
///     pub fn new() -> Self {
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
