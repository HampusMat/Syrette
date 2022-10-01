use std::error::Error;

use quote::{format_ident, quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::{parse_str, ExprMethodCall, FnArg, Generics, ItemImpl, Type};

use crate::injectable::dependency::Dependency;
use crate::util::item_impl::find_impl_method_by_name_mut;
use crate::util::string::camelcase_to_snakecase;
use crate::util::syn_path::syn_path_to_string;

const DI_CONTAINER_VAR_NAME: &str = "di_container";
const DEPENDENCY_HISTORY_VAR_NAME: &str = "dependency_history";

pub struct InjectableImpl
{
    pub dependencies: Vec<Dependency>,
    pub self_type: Type,
    pub generics: Generics,
    pub original_impl: ItemImpl,
}

impl Parse for InjectableImpl
{
    fn parse(input: ParseStream) -> syn::Result<Self>
    {
        let mut impl_parsed_input = input.parse::<ItemImpl>()?;

        let dependencies = Self::build_dependencies(&mut impl_parsed_input)
            .map_err(|err| input.error(err))?;

        Ok(Self {
            dependencies,
            self_type: impl_parsed_input.self_ty.as_ref().clone(),
            generics: impl_parsed_input.generics.clone(),
            original_impl: impl_parsed_input,
        })
    }
}

impl InjectableImpl
{
    pub fn expand(&self, no_doc_hidden: bool, is_async: bool)
        -> proc_macro2::TokenStream
    {
        let Self {
            dependencies,
            self_type,
            generics,
            original_impl,
        } = self;

        let di_container_var = format_ident!("{}", DI_CONTAINER_VAR_NAME);
        let dependency_history_var = format_ident!("{}", DEPENDENCY_HISTORY_VAR_NAME);

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

        let injectable_impl = if is_async {
            let async_get_dep_method_calls =
                Self::create_get_dep_method_calls(dependencies, true);

            quote! {
                #maybe_doc_hidden
                impl #generics syrette::interfaces::async_injectable::AsyncInjectable for #self_type
                {
                    fn resolve<'di_container, 'fut>(
                        #di_container_var: &'di_container std::sync::Arc<
                            syrette::async_di_container::AsyncDIContainer
                        >,
                        mut #dependency_history_var: Vec<&'static str>,
                    ) -> syrette::future::BoxFuture<
                        'fut,
                        Result<
                            syrette::ptr::TransientPtr<Self>,
                            syrette::errors::injectable::InjectableError
                        >
                    >
                    where
                        Self: Sized + 'fut,
                        'di_container: 'fut
                    {
                        Box::pin(async move {
                            use std::any::type_name;

                            use syrette::errors::injectable::InjectableError;

                            let self_type_name = type_name::<#self_type>();

                            #maybe_prevent_circular_deps

                            Ok(syrette::ptr::TransientPtr::new(Self::new(
                                #(#async_get_dep_method_calls),*
                            )))
                        })
                    }
                }

            }
        } else {
            let get_dep_method_calls =
                Self::create_get_dep_method_calls(dependencies, false);

            quote! {
                #maybe_doc_hidden
                impl #generics syrette::interfaces::injectable::Injectable for #self_type
                {
                    fn resolve(
                        #di_container_var: &std::rc::Rc<syrette::DIContainer>,
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
        };

        quote! {
            #original_impl

            #injectable_impl
        }
    }

    fn create_get_dep_method_calls(
        dependencies: &[Dependency],
        is_async: bool,
    ) -> Vec<proc_macro2::TokenStream>
    {
        dependencies
            .iter()
            .filter_map(|dependency| {
                let dep_interface_str = match &dependency.interface {
                    Type::TraitObject(interface_trait) => {
                        Some(interface_trait.to_token_stream().to_string())
                    }
                    Type::Path(path_interface) => {
                        Some(syn_path_to_string(&path_interface.path))
                    }
                    &_ => None,
                }?;

                let method_call = parse_str::<ExprMethodCall>(
                    format!(
                        "{}.get_bound::<{}>({}.clone(), {})",
                        DI_CONTAINER_VAR_NAME,
                        dep_interface_str,
                        DEPENDENCY_HISTORY_VAR_NAME,
                        dependency.name.as_ref().map_or_else(
                            || "None".to_string(),
                            |name| format!("Some(\"{}\")", name.value())
                        )
                    )
                    .as_str(),
                )
                .ok()?;

                Some((method_call, dependency))
            })
            .map(|(method_call, dep_type)| {
                let ptr_name = dep_type.ptr.to_string();

                let to_ptr = format_ident!(
                    "{}",
                    camelcase_to_snakecase(&ptr_name.replace("Ptr", ""))
                );

                let do_method_call = if is_async {
                    quote! { #method_call.await }
                } else {
                    quote! { #method_call }
                };

                let resolve_failed_error = if is_async {
                    quote! { InjectableError::AsyncResolveFailed }
                } else {
                    quote! { InjectableError::ResolveFailed }
                };

                quote! {
                    #do_method_call.map_err(|err| #resolve_failed_error {
                        reason: Box::new(err),
                        affected: self_type_name
                    })?.#to_ptr().unwrap()
                }
            })
            .collect()
    }

    fn build_dependencies(
        item_impl: &mut ItemImpl,
    ) -> Result<Vec<Dependency>, Box<dyn Error>>
    {
        let new_method_impl_item = find_impl_method_by_name_mut(item_impl, "new")
            .map_or_else(|| Err("Missing a 'new' method"), Ok)?;

        let new_method_args = &mut new_method_impl_item.sig.inputs;

        let dependencies: Result<Vec<_>, _> =
            new_method_args.iter().map(Dependency::build).collect();

        for arg in new_method_args {
            let typed_arg = if let FnArg::Typed(typed_arg) = arg {
                typed_arg
            } else {
                continue;
            };

            let attrs_to_remove: Vec<_> = typed_arg
                .attrs
                .iter()
                .enumerate()
                .filter_map(|(index, attr)| {
                    if syn_path_to_string(&attr.path).as_str() == "syrette::named" {
                        return Some(index);
                    }

                    if attr.path.get_ident()?.to_string().as_str() == "named" {
                        return Some(index);
                    }

                    None
                })
                .collect();

            for attr_index in attrs_to_remove {
                typed_arg.attrs.remove(attr_index);
            }
        }

        dependencies
    }
}
