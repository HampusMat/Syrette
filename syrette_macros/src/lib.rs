use proc_macro::TokenStream;
use quote::quote;
use syn::{parse, parse_macro_input};

mod factory_type_alias;
mod injectable_impl;
mod injectable_macro_args;
mod libs;

use factory_type_alias::FactoryTypeAlias;
use injectable_impl::InjectableImpl;
use injectable_macro_args::InjectableMacroArgs;
use libs::intertrait_macros::{
    args::{Casts, Flag, Targets},
    gen_caster::generate_caster,
};

/// Makes a struct injectable. Thereby usable with `DIContainer`.
///
/// # Arguments
///
/// * A interface trait the struct implements.
///
/// # Examples
/// ```
/// trait IConfigReader
/// {
///     fn read_config(&self) -> Config;
/// }
///
/// struct ConfigReader
/// {
///     _file_reader: InterfacePtr<IFileReader>,
/// }
///
/// impl ConfigReader
/// {
///     fn new(file_reader: InterfacePtr<IFileReader>) -> Self
///     {
///         Self {
///             _file_reader: file_reader
///         }
///     }
/// }
///
/// #[injectable(IConfigReader)]
/// impl IConfigReader for ConfigReader
/// {
///     fn read_config(&self) -> Config
///     {
///         // Stuff here
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn injectable(args_stream: TokenStream, impl_stream: TokenStream) -> TokenStream
{
    let InjectableMacroArgs {
        interface: interface_type_path,
    } = parse_macro_input!(args_stream);

    let injectable_impl: InjectableImpl = parse(impl_stream).unwrap();

    let expanded_injectable_impl = injectable_impl.expand();

    let self_type = &injectable_impl.self_type;

    quote! {
        #expanded_injectable_impl

        syrette::castable_to!(#self_type => #interface_type_path);
    }
    .into()
}

/// Makes a type alias usable as a factory interface.
///
/// # Examples
/// ```
/// trait IUser
/// {
///     fn name(&self) -> String;
///     fn age(&self) -> i32;
/// }
///
/// struct User
/// {
///     _name: String,
///     _age: i32,
/// }
///
/// impl User
/// {
///     fn new(name: String, age: i32) -> Self
///     {
///         Self {
///             _name: name,
///             _age: age,
///         }
///     }
/// }
///
/// impl IUser for User
/// {
///     fn name(&self) -> String
///     {
///         self._name
///     }
///
///     fn age(&self) -> i32
///     {
///         self._age
///     }
/// }
///
/// type UserFactory = dyn IFactory<(String, i32), dyn IUser>;
/// ```
#[proc_macro_attribute]
pub fn factory(_: TokenStream, type_alias_stream: TokenStream) -> TokenStream
{
    let FactoryTypeAlias {
        type_alias,
        factory_interface,
        arg_types,
        return_type,
    } = parse(type_alias_stream).unwrap();

    quote! {
        #type_alias

        syrette::castable_to!(
            syrette::castable_factory::CastableFactory<
                #arg_types,
                #return_type
            > => #factory_interface
        );

        syrette::castable_to!(
            syrette::castable_factory::CastableFactory<
                #arg_types,
                #return_type
            > => syrette::castable_factory::AnyFactory
        );
    }
    .into()
}

#[doc(hidden)]
#[proc_macro]
pub fn castable_to(input: TokenStream) -> TokenStream
{
    let Casts {
        ty,
        targets: Targets { flags, paths },
    } = parse_macro_input!(input);

    paths
        .iter()
        .map(|t| generate_caster(&ty, t, flags.contains(&Flag::Sync)))
        .collect::<proc_macro2::TokenStream>()
        .into()
}
