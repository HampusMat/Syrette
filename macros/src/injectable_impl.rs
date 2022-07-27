use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::Generics;
use syn::{
    parse_str, punctuated::Punctuated, token::Comma, ExprMethodCall, FnArg, Ident,
    ImplItem, ImplItemMethod, ItemImpl, Path, Type, TypePath,
};

use crate::dependency_type::DependencyType;

const DI_CONTAINER_VAR_NAME: &str = "di_container";

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

        let dependency_types = Self::_get_dependency_types(&impl_parsed_input)
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
    pub fn expand(&self) -> proc_macro2::TokenStream
    {
        let Self {
            dependency_types,
            self_type,
            generics,
            original_impl,
        } = self;

        let di_container_var: Ident = parse_str(DI_CONTAINER_VAR_NAME).unwrap();

        let get_dependencies = Self::_create_get_dependencies(dependency_types);

        quote! {
            #original_impl

            impl #generics syrette::interfaces::injectable::Injectable for #self_type {
                fn resolve(
                    #di_container_var: &syrette::DIContainer
                ) -> syrette::libs::error_stack::Result<
                    syrette::ptr::TransientPtr<Self>,
                    syrette::errors::injectable::ResolveError>
                {
                    use syrette::libs::error_stack::ResultExt;

                    return Ok(syrette::ptr::TransientPtr::new(Self::new(
                        #(#get_dependencies
                            .change_context(syrette::errors::injectable::ResolveError)
                            .attach_printable(
                                format!(
                                    "Unable to resolve a dependency of {}",
                                    std::any::type_name::<#self_type>()
                                )
                            )?
                        ),*
                    )));
                }
            }
        }
    }

    fn _create_get_dependencies(
        dependency_types: &[DependencyType],
    ) -> Vec<ExprMethodCall>
    {
        dependency_types
            .iter()
            .filter_map(|dep_type| match &dep_type.interface {
                Type::TraitObject(dep_type_trait) => Some(
                    parse_str(
                        format!(
                            "{}.get{}::<{}>()",
                            DI_CONTAINER_VAR_NAME,
                            if dep_type.ptr == "SingletonPtr" {
                                "_singleton"
                            } else {
                                ""
                            },
                            dep_type_trait.to_token_stream()
                        )
                        .as_str(),
                    )
                    .unwrap(),
                ),
                Type::Path(dep_type_path) => {
                    let dep_type_path_str = Self::_path_to_string(&dep_type_path.path);

                    let get_method_name = if dep_type_path_str.ends_with("Factory") {
                        "get_factory"
                    } else {
                        "get"
                    };

                    Some(
                        parse_str(
                            format!(
                                "{}.{}::<{}>()",
                                DI_CONTAINER_VAR_NAME, get_method_name, dep_type_path_str
                            )
                            .as_str(),
                        )
                        .unwrap(),
                    )
                }
                &_ => None,
            })
            .collect()
    }

    fn _find_method_by_name<'impl_lt>(
        item_impl: &'impl_lt ItemImpl,
        method_name: &'static str,
    ) -> Option<&'impl_lt ImplItemMethod>
    {
        let impl_items = &item_impl.items;

        impl_items
            .iter()
            .filter_map(|impl_item| match impl_item {
                ImplItem::Method(method_item) => Some(method_item),
                &_ => None,
            })
            .find(|method_item| method_item.sig.ident == method_name)
    }

    #[allow(clippy::match_wildcard_for_single_variants)]
    fn get_has_fn_args_self(fn_args: &Punctuated<FnArg, Comma>) -> bool
    {
        fn_args.iter().any(|arg| match arg {
            FnArg::Receiver(_) => true,
            &_ => false,
        })
    }

    fn _get_fn_arg_type_paths(fn_args: &Punctuated<FnArg, Comma>) -> Vec<&TypePath>
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

    fn _path_to_string(path: &Path) -> String
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

    fn _is_type_path_ptr(type_path: &TypePath) -> bool
    {
        let arg_type_path_string = Self::_path_to_string(&type_path.path);

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

    fn _get_dependency_types(
        item_impl: &ItemImpl,
    ) -> Result<Vec<DependencyType>, &'static str>
    {
        let new_method_impl_item = match Self::_find_method_by_name(item_impl, "new") {
            Some(method_item) => Ok(method_item),
            None => Err("Missing a 'new' method"),
        }?;

        let new_method_args = &new_method_impl_item.sig.inputs;

        if Self::get_has_fn_args_self(new_method_args) {
            return Err("Unexpected self argument in 'new' method");
        }

        let new_method_arg_type_paths = Self::_get_fn_arg_type_paths(new_method_args);

        if new_method_arg_type_paths
            .iter()
            .any(|arg_type_path| !Self::_is_type_path_ptr(arg_type_path))
        {
            return Err("All argument types in 'new' method must ptr types");
        }

        Ok(new_method_arg_type_paths
            .iter()
            .filter_map(|arg_type_path| DependencyType::from_type_path(arg_type_path))
            .collect())
    }
}
