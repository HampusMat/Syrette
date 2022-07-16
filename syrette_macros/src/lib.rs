use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse, parse_macro_input, parse_str, punctuated::Punctuated, token::Comma,
    AttributeArgs, ExprMethodCall, FnArg, GenericArgument, ImplItem, ItemImpl, ItemType,
    Meta, NestedMeta, Path, PathArguments, Type, TypeParamBound, TypePath,
};

mod libs;

use libs::intertrait_macros::{
    args::{Casts, Flag, Targets},
    gen_caster::generate_caster,
};

const NO_INTERFACE_ARG_ERR_MESSAGE: &str =
    "Expected a argument specifying a interface trait";

const INVALID_ARG_ERR_MESSAGE: &str = "Invalid argument passed";

const INVALID_ITEM_TYPE_ERR_MESSAGE: &str =
    "The attached to item is not a trait implementation";

const IMPL_NO_NEW_METHOD_ERR_MESSAGE: &str =
    "The attached to trait implementation is missing a new method";

const IMPL_NEW_METHOD_SELF_PARAM_ERR_MESSAGE: &str =
    "The new method of the attached to trait implementation cannot have a self parameter";

const IMPL_NEW_METHOD_PARAM_TYPES_ERR_MESSAGE: &str = concat!(
    "All parameters of the new method of the attached to trait implementation ",
    "must be either syrette::ptr::InterfacePtr or syrrete::ptr::FactoryPtr (for factories)"
);

const INVALID_ALIASED_FACTORY_TRAIT_ERR_MESSAGE: &str =
    "Invalid aliased trait. Must be 'dyn IFactory'";

const INVALID_ALIASED_FACTORY_ARGS_ERR_MESSAGE: &str =
    "Invalid arguments for 'dyn IFactory'";

fn path_to_string(path: &Path) -> String
{
    return path
        .segments
        .pairs()
        .fold(String::new(), |mut acc, segment_pair| {
            let segment_ident = &segment_pair.value().ident;

            acc.push_str(segment_ident.to_string().as_str());

            let opt_colon_two = segment_pair.punct();

            match opt_colon_two {
                Some(colon_two) => {
                    acc.push_str(colon_two.to_token_stream().to_string().as_str())
                }
                None => {}
            }

            acc
        });
}

fn get_fn_args_has_self(fn_args: &Punctuated<FnArg, Comma>) -> bool
{
    return fn_args.iter().any(|arg| match arg {
        FnArg::Receiver(_) => true,
        &_ => false,
    });
}

fn get_fn_arg_type_paths(fn_args: &Punctuated<FnArg, Comma>) -> Vec<TypePath>
{
    return fn_args.iter().fold(Vec::<TypePath>::new(), |mut acc, arg| {
        match arg {
            FnArg::Typed(typed_fn_arg) => match typed_fn_arg.ty.as_ref() {
                Type::Path(arg_type_path) => acc.push(arg_type_path.clone()),
                Type::Reference(ref_type_path) => match ref_type_path.elem.as_ref() {
                    Type::Path(arg_type_path) => acc.push(arg_type_path.clone()),
                    &_ => {}
                },
                &_ => {}
            },
            FnArg::Receiver(_receiver_fn_arg) => {}
        }

        acc
    });
}

fn get_dependency_types(item_impl: &ItemImpl) -> Vec<Type>
{
    let impl_items = &item_impl.items;

    let opt_new_method_impl_item = impl_items.iter().find(|item| match item {
        ImplItem::Method(method_item) => method_item.sig.ident == "new",
        &_ => false,
    });

    let new_method_impl_item = match opt_new_method_impl_item {
        Some(item) => match item {
            ImplItem::Method(method_item) => method_item,
            &_ => panic!("{}", IMPL_NO_NEW_METHOD_ERR_MESSAGE),
        },
        None => panic!("{}", IMPL_NO_NEW_METHOD_ERR_MESSAGE),
    };

    let new_method_inputs = &new_method_impl_item.sig.inputs;

    if get_fn_args_has_self(new_method_inputs) {
        panic!("{}", IMPL_NEW_METHOD_SELF_PARAM_ERR_MESSAGE)
    }

    let new_method_arg_type_paths = get_fn_arg_type_paths(new_method_inputs);

    return new_method_arg_type_paths.iter().fold(
        Vec::<Type>::new(),
        |mut acc, arg_type_path| {
            let arg_type_path_string = path_to_string(&arg_type_path.path);

            if arg_type_path_string != "InterfacePtr"
                && arg_type_path_string != "ptr::InterfacePtr"
                && arg_type_path_string != "syrrete::ptr::InterfacePtr"
                && arg_type_path_string != "FactoryPtr"
                && arg_type_path_string != "ptr::FactoryPtr"
                && arg_type_path_string != "syrrete::ptr::FactoryPtr"
            {
                panic!("{}", IMPL_NEW_METHOD_PARAM_TYPES_ERR_MESSAGE);
            }

            // Assume the type path has a last segment.
            let last_path_segment = arg_type_path.path.segments.last().unwrap();

            match &last_path_segment.arguments {
                PathArguments::AngleBracketed(angle_bracketed_generic_args) => {
                    let generic_args = &angle_bracketed_generic_args.args;

                    let opt_first_generic_arg = generic_args.first();

                    // Assume a first generic argument exists because InterfacePtr and
                    // FactoryPtr requires one
                    let first_generic_arg = opt_first_generic_arg.as_ref().unwrap();

                    match first_generic_arg {
                        GenericArgument::Type(first_generic_arg_type) => {
                            acc.push(first_generic_arg_type.clone());
                        }
                        &_ => {}
                    }
                }
                &_ => {}
            }

            acc
        },
    );
}

/// Makes a struct injectable. Therefore usable with `DIContainer`.
///
/// # Arguments
///
/// * A interface trait the struct implements.
///
/// # Examples
/// ```
/// trait IConfigReader
/// {
///     fn read_config() -> Config;
/// }
///
/// struct ConfigReader {}
///
/// #[injectable(IConfigReader)]
/// impl IConfigReader for ConfigReader
/// {
///     fn read_config() -> Config
///     {
///         // Stuff here
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn injectable(args_stream: TokenStream, impl_stream: TokenStream) -> TokenStream
{
    let args = parse_macro_input!(args_stream as AttributeArgs);

    if args.is_empty() {
        panic!("{}", NO_INTERFACE_ARG_ERR_MESSAGE);
    }

    if args.len() > 1 {
        panic!("Only a single argument is expected");
    }

    let interface_path = match &args[0] {
        NestedMeta::Meta(arg_meta) => match arg_meta {
            Meta::Path(path_arg) => path_arg,
            &_ => panic!("{}", INVALID_ARG_ERR_MESSAGE),
        },
        &_ => panic!("{}", INVALID_ARG_ERR_MESSAGE),
    };

    let item_impl: ItemImpl = match parse(impl_stream) {
        Ok(impl_parsed) => impl_parsed,
        Err(_) => {
            panic!("{}", INVALID_ITEM_TYPE_ERR_MESSAGE)
        }
    };

    let self_type = item_impl.self_ty.as_ref();

    let self_type_path = match self_type {
        Type::Path(path_self_type) => path_self_type.path.clone(),
        &_ => parse_str("invalid_type").unwrap(),
    };

    let dependency_types = get_dependency_types(&item_impl);

    let get_dependencies = dependency_types.iter().fold(
        Vec::<ExprMethodCall>::new(),
        |mut acc, dep_type| {
            match dep_type {
                Type::TraitObject(dep_type_trait) => {
                    acc.push(
                        parse_str(
                            format!(
                                "di_container.get::<{}>()",
                                dep_type_trait.to_token_stream()
                            )
                            .as_str(),
                        )
                        .unwrap(),
                    );
                }
                Type::Path(dep_type_path) => {
                    let dep_type_path_str = path_to_string(&dep_type_path.path);

                    let get_method_name = if dep_type_path_str.ends_with("Factory") {
                        "get_factory"
                    } else {
                        "get"
                    };

                    acc.push(
                        parse_str(
                            format!(
                                "di_container.{}::<{}>()",
                                get_method_name, dep_type_path_str
                            )
                            .as_str(),
                        )
                        .unwrap(),
                    );
                }
                &_ => {}
            }

            acc
        },
    );

    quote! {
        #item_impl

        impl syrette::interfaces::injectable::Injectable for #self_type_path {
            fn resolve(
                di_container: &syrette::DIContainer
            ) -> error_stack::Result<
                syrette::ptr::InterfacePtr<Self>,
                syrette::errors::injectable::ResolveError>
            {
                use error_stack::ResultExt;

                return Ok(syrette::ptr::InterfacePtr::new(Self::new(
                    #(#get_dependencies
                        .change_context(syrette::errors::injectable::ResolveError)
                        .attach_printable(
                            format!(
                                "Unable to resolve a dependency of {}",
                                std::any::type_name::<#self_type_path>()
                            )
                        )?
                    ),*
                )));
            }
        }

        syrette::castable_to!(#self_type_path => #interface_path);
    }
    .into()
}

#[proc_macro_attribute]
pub fn factory(_: TokenStream, type_alias_stream: TokenStream) -> TokenStream
{
    let type_alias: ItemType = parse(type_alias_stream).unwrap();

    let aliased_trait = match &type_alias.ty.as_ref() {
        Type::TraitObject(alias_type) => alias_type,
        &_ => panic!("{}", INVALID_ALIASED_FACTORY_TRAIT_ERR_MESSAGE),
    };

    if aliased_trait.bounds.len() != 1 {
        panic!("{}", INVALID_ALIASED_FACTORY_TRAIT_ERR_MESSAGE);
    }

    let type_bound = aliased_trait.bounds.first().unwrap();

    let trait_bound = match type_bound {
        TypeParamBound::Trait(trait_bound) => trait_bound,
        &_ => panic!("{}", INVALID_ALIASED_FACTORY_TRAIT_ERR_MESSAGE),
    };

    let trait_bound_path = &trait_bound.path;

    if trait_bound_path.segments.is_empty()
        || trait_bound_path.segments.last().unwrap().ident != "IFactory"
    {
        panic!("{}", INVALID_ALIASED_FACTORY_TRAIT_ERR_MESSAGE);
    }

    let factory_path_segment = trait_bound_path.segments.last().unwrap();

    let factory_path_segment_args = &match &factory_path_segment.arguments {
        syn::PathArguments::AngleBracketed(args) => args,
        &_ => panic!("{}", INVALID_ALIASED_FACTORY_ARGS_ERR_MESSAGE),
    }
    .args;

    let factory_arg_types_type = match &factory_path_segment_args[0] {
        GenericArgument::Type(arg_type) => arg_type,
        &_ => panic!("{}", INVALID_ALIASED_FACTORY_ARGS_ERR_MESSAGE),
    };

    let factory_return_type = match &factory_path_segment_args[1] {
        GenericArgument::Type(arg_type) => arg_type,
        &_ => panic!("{}", INVALID_ALIASED_FACTORY_ARGS_ERR_MESSAGE),
    };

    quote! {
        #type_alias

        syrette::castable_to!(
            syrette::castable_factory::CastableFactory<
                #factory_arg_types_type,
                #factory_return_type
            > => #trait_bound_path
        );

        syrette::castable_to!(
            syrette::castable_factory::CastableFactory<
                #factory_arg_types_type,
                #factory_return_type
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
