use proc_macro::TokenStream;
use quote::quote;
use syn::{parse, parse_macro_input};

mod declare_interface_args;
mod factory_type_alias;
mod injectable_impl;
mod injectable_macro_args;
mod libs;

use declare_interface_args::DeclareInterfaceArgs;
use factory_type_alias::FactoryTypeAlias;
use injectable_impl::InjectableImpl;
use injectable_macro_args::InjectableMacroArgs;
use libs::intertrait_macros::gen_caster::generate_caster;

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

        syrette::declare_interface!(#self_type -> #interface_type_path);
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
            > -> syrette::castable_factory::AnyFactory
        );
    }
    .into()
}

/// Declares the interface trait of a implementation.
///
/// # Examples
/// ```
/// declare_interface!(ClientService -> IClientService);
///
/// ```
///
/// With `ClientService` in this case being the concrete
/// implementation and `IClientService` being the interface trait.
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
