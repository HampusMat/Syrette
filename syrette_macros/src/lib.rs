use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse, parse_macro_input, parse_str, punctuated::Punctuated, token::Comma,
    AttributeArgs, FnArg, GenericArgument, ImplItem, ItemImpl, Meta, NestedMeta, Path,
    PathArguments, Type, TypePath,
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

const IMPL_NEW_METHOD_BOX_PARAMS_ERR_MESSAGE: &str =
    "All parameters of the new method of the attached to trait implementation must be std::boxed::Box";

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

            if arg_type_path_string != "Box"
                && arg_type_path_string != "std::boxed::Box"
                && arg_type_path_string != "boxed::Box"
            {
                panic!("{}", IMPL_NEW_METHOD_BOX_PARAMS_ERR_MESSAGE);
            }

            // Assume the type path has a last segment.
            // The Box check wouldn't pass if it didn't
            let last_path_segment = arg_type_path.path.segments.last().unwrap();

            match &last_path_segment.arguments {
                PathArguments::AngleBracketed(angle_bracketed_generic_args) => {
                    let generic_args = &angle_bracketed_generic_args.args;

                    let opt_first_generic_arg = generic_args.first();

                    // Assume a first generic argument exists because Box requires one
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

    quote! {
        #item_impl

        impl syrette::injectable::Injectable for #self_type_path {
            fn resolve(
                di_container: &syrette::DIContainer
            ) -> error_stack::Result<Box<Self>, syrette::injectable::ResolveError>
            {
                use error_stack::ResultExt;

                return Ok(Box::new(Self::new(
                    #(di_container.get::<#dependency_types>()
                        .change_context(syrette::injectable::ResolveError)
                        .attach_printable(format!(
                            "Unable to resolve a dependency of {}",
                            std::any::type_name::<#self_type_path>()
                        ))?,
                    )*
                )));
            }
        }

        syrette::castable_to!(#self_type_path => #interface_path);
    }
    .into()
}

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
