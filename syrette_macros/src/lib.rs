#![deny(clippy::all)]
#![deny(clippy::pedantic)]

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
/// * A interface trait the struct implements.
///
/// # Panics
/// If the attributed item is not a impl.
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
/// # Panics
/// If the attributed item is not a type alias.
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
