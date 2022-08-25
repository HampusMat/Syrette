use quote::{format_ident, quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::Generics;
use syn::{
    parse_str, punctuated::Punctuated, token::Comma, ExprMethodCall, FnArg, ItemImpl,
    Path, Type, TypePath,
};

use crate::dependency_type::DependencyType;
use crate::util::item_impl::find_impl_method_by_name;

const DI_CONTAINER_VAR_NAME: &str = "di_container";
const DEPENDENCY_HISTORY_VAR_NAME: &str = "dependency_history";

pub struct InjectableImpl
{
    pub dependency_types: Vec<DependencyType>,
    pub self_type: Type,
    pub generics: Generics,
    pub original_impl: ItemImpl,
}

impl Parse for InjectableImpl
{
    fn parse(input: ParseStream) -> syn::Result<Self>
    {
        let impl_parsed_input = input.parse::<ItemImpl>()?;

        let dependency_types = Self::get_dependency_types(&impl_parsed_input)
            .map_err(|err| input.error(err))?;

        Ok(Self {
            dependency_types,
            self_type: impl_parsed_input.self_ty.as_ref().clone(),
            generics: impl_parsed_input.generics.clone(),
            original_impl: impl_parsed_input,
        })
    }
}

impl InjectableImpl
{
    pub fn expand(&self, no_doc_hidden: bool) -> proc_macro2::TokenStream
    {
        let Self {
            dependency_types,
            self_type,
            generics,
            original_impl,
        } = self;

        let di_container_var = format_ident!("{}", DI_CONTAINER_VAR_NAME);
        let dependency_history_var = format_ident!("{}", DEPENDENCY_HISTORY_VAR_NAME);

        let get_dep_method_calls = Self::create_get_dep_method_calls(dependency_types);

        let maybe_doc_hidden = if no_doc_hidden {
            quote! {}
        } else {
            quote! {
                #[doc(hidden)]
            }
        };

        let maybe_prevent_circular_deps = if cfg!(feature = "prevent-circular") {
            quote! {
                if #dependency_history_var.contains(&self_type_name) {
                    #dependency_history_var.push(self_type_name);

                    let dependency_trace =
                        syrette::dependency_trace::create_dependency_trace(
                            #dependency_history_var.as_slice(),
                            self_type_name
                        );

                    return Err(InjectableError::DetectedCircular {dependency_trace });
                }

                #dependency_history_var.push(self_type_name);
            }
        } else {
            quote! {}
        };

        quote! {
            #original_impl

            #maybe_doc_hidden
            impl #generics syrette::interfaces::injectable::Injectable for #self_type {
                fn resolve(
                    #di_container_var: &syrette::DIContainer,
                    mut #dependency_history_var: Vec<&'static str>,
                ) -> Result<
                    syrette::ptr::TransientPtr<Self>,
                    syrette::errors::injectable::InjectableError>
                {
                    use std::any::type_name;

                    use syrette::errors::injectable::InjectableError;

                    let self_type_name = type_name::<#self_type>();

                    #maybe_prevent_circular_deps

                    return Ok(syrette::ptr::TransientPtr::new(Self::new(
                        #(#get_dep_method_calls),*
                    )));
                }
            }
        }
    }

    fn create_get_dep_method_calls(
        dependency_types: &[DependencyType],
    ) -> Vec<proc_macro2::TokenStream>
    {
        dependency_types
            .iter()
            .filter_map(|dep_type| match &dep_type.interface {
                Type::TraitObject(dep_type_trait) => {
                    let method_call = parse_str::<ExprMethodCall>(
                        format!(
                            "{}.get_bound::<{}>({}.clone())",
                            DI_CONTAINER_VAR_NAME,
                            dep_type_trait.to_token_stream(),
                            DEPENDENCY_HISTORY_VAR_NAME
                        )
                        .as_str(),
                    )
                    .ok()?;

                    Some((method_call, dep_type))

                    /*
                     */
                }
                Type::Path(dep_type_path) => {
                    let dep_type_path_str = Self::path_to_string(&dep_type_path.path);

                    let method_call: ExprMethodCall = parse_str(
                        format!(
                            "{}.get_bound::<{}>({}.clone())",
                            DI_CONTAINER_VAR_NAME,
                            dep_type_path_str,
                            DEPENDENCY_HISTORY_VAR_NAME
                        )
                        .as_str(),
                    )
                    .ok()?;

                    Some((method_call, dep_type))
                }
                &_ => None,
            })
            .map(|(method_call, dep_type)| {
                let ptr_name = dep_type.ptr.to_string();

                let to_ptr =
                    format_ident!("{}", ptr_name.replace("Ptr", "").to_lowercase());

                quote! {
                    #method_call.map_err(|err| InjectableError::ResolveFailed {
                        reason: Box::new(err),
                        affected: self_type_name
                    })?.#to_ptr().unwrap()
                }
            })
            .collect()
    }

    #[allow(clippy::match_wildcard_for_single_variants)]
    fn get_has_fn_args_self(fn_args: &Punctuated<FnArg, Comma>) -> bool
    {
        fn_args.iter().any(|arg| match arg {
            FnArg::Receiver(_) => true,
            &_ => false,
        })
    }

    fn get_fn_arg_type_paths(fn_args: &Punctuated<FnArg, Comma>) -> Vec<&TypePath>
    {
        fn_args
            .iter()
            .filter_map(|arg| match arg {
                FnArg::Typed(typed_fn_arg) => match typed_fn_arg.ty.as_ref() {
                    Type::Path(arg_type_path) => Some(arg_type_path),
                    Type::Reference(ref_type_path) => match ref_type_path.elem.as_ref() {
                        Type::Path(arg_type_path) => Some(arg_type_path),
                        &_ => None,
                    },
                    &_ => None,
                },
                FnArg::Receiver(_receiver_fn_arg) => None,
            })
            .collect()
    }

    fn path_to_string(path: &Path) -> String
    {
        path.segments
            .pairs()
            .fold(String::new(), |mut acc, segment_pair| {
                let segment_ident = &segment_pair.value().ident;

                acc.push_str(segment_ident.to_string().as_str());

                let opt_colon_two = segment_pair.punct();

                match opt_colon_two {
                    Some(colon_two) => {
                        acc.push_str(colon_two.to_token_stream().to_string().as_str());
                    }
                    None => {}
                }

                acc
            })
    }

    fn is_type_path_ptr(type_path: &TypePath) -> bool
    {
        let arg_type_path_string = Self::path_to_string(&type_path.path);

        arg_type_path_string == "TransientPtr"
            || arg_type_path_string == "ptr::TransientPtr"
            || arg_type_path_string == "syrrete::ptr::TransientPtr"
            || arg_type_path_string == "SingletonPtr"
            || arg_type_path_string == "ptr::SingletonPtr"
            || arg_type_path_string == "syrrete::ptr::SingletonPtr"
            || arg_type_path_string == "FactoryPtr"
            || arg_type_path_string == "ptr::FactoryPtr"
            || arg_type_path_string == "syrrete::ptr::FactoryPtr"
    }

    fn get_dependency_types(
        item_impl: &ItemImpl,
    ) -> Result<Vec<DependencyType>, &'static str>
    {
        let new_method_impl_item = find_impl_method_by_name(item_impl, "new")
            .map_or_else(|| Err("Missing a 'new' method"), Ok)?;

        let new_method_args = &new_method_impl_item.sig.inputs;

        if Self::get_has_fn_args_self(new_method_args) {
            return Err("Unexpected self argument in 'new' method");
        }

        let new_method_arg_type_paths = Self::get_fn_arg_type_paths(new_method_args);

        if new_method_arg_type_paths
            .iter()
            .any(|arg_type_path| !Self::is_type_path_ptr(arg_type_path))
        {
            return Err("All argument types in 'new' method must ptr types");
        }

        Ok(new_method_arg_type_paths
            .iter()
            .filter_map(|arg_type_path| DependencyType::from_type_path(arg_type_path))
            .collect())
    }
}
