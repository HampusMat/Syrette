use std::error::Error;

use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::spanned::Spanned;
use syn::{
    parse2,
    parse_str,
    ExprMethodCall,
    FnArg,
    Generics,
    ImplItemMethod,
    ItemImpl,
    ReturnType,
    Type,
};

use crate::injectable::dependency::{DependencyError, IDependency};
use crate::util::error::diagnostic_error_enum;
use crate::util::item_impl::find_impl_method_by_name_mut;
use crate::util::string::camelcase_to_snakecase;
use crate::util::syn_path::SynPathExt;

const DI_CONTAINER_VAR_NAME: &str = "di_container";
const DEPENDENCY_HISTORY_VAR_NAME: &str = "dependency_history";

pub struct InjectableImpl<Dep: IDependency>
{
    pub dependencies: Vec<Dep>,
    pub self_type: Type,
    pub generics: Generics,
    pub original_impl: ItemImpl,

    new_method: ImplItemMethod,
}

impl<Dep: IDependency> InjectableImpl<Dep>
{
    #[cfg(not(tarpaulin_include))]
    pub fn parse(input: TokenStream) -> Result<Self, InjectableImplError>
    {
        let mut item_impl = parse2::<ItemImpl>(input).map_err(|err| {
            InjectableImplError::NotAImplementation {
                err_span: err.span(),
            }
        })?;

        if let Some((_, trait_path, _)) = item_impl.trait_ {
            return Err(InjectableImplError::TraitImpl {
                trait_path_span: trait_path.span(),
            });
        }

        let item_impl_span = item_impl.self_ty.span();

        let new_method = find_impl_method_by_name_mut(&mut item_impl, "new").ok_or(
            InjectableImplError::MissingNewMethod {
                implementation_span: item_impl_span,
            },
        )?;

        let dependencies = Self::build_dependencies(new_method).map_err(|err| {
            InjectableImplError::ContainsAInvalidDependency {
                implementation_span: item_impl_span,
                err,
            }
        })?;

        Self::remove_method_argument_attrs(new_method);

        let new_method = new_method.clone();

        Ok(Self {
            dependencies,
            self_type: item_impl.self_ty.as_ref().clone(),
            generics: item_impl.generics.clone(),
            original_impl: item_impl,
            new_method,
        })
    }

    pub fn validate(&self) -> Result<(), InjectableImplError>
    {
        if matches!(self.new_method.sig.output, ReturnType::Default) {
            return Err(InjectableImplError::InvalidNewMethodReturnType {
                new_method_output_span: self.new_method.sig.output.span(),
                expected: "Self".to_string(),
                found: "()".to_string(),
            });
        }

        if let ReturnType::Type(_, ret_type) = &self.new_method.sig.output {
            if let Type::Path(path_type) = ret_type.as_ref() {
                if path_type
                    .path
                    .get_ident()
                    .map_or_else(|| true, |ident| *ident != "Self")
                {
                    return Err(InjectableImplError::InvalidNewMethodReturnType {
                        new_method_output_span: self.new_method.sig.output.span(),
                        expected: "Self".to_string(),
                        found: ret_type.to_token_stream().to_string(),
                    });
                }
            } else {
                return Err(InjectableImplError::InvalidNewMethodReturnType {
                    new_method_output_span: self.new_method.sig.output.span(),
                    expected: "Self".to_string(),
                    found: ret_type.to_token_stream().to_string(),
                });
            }
        }

        if let Some(unsafety) = self.new_method.sig.unsafety {
            return Err(InjectableImplError::NewMethodUnsafe {
                unsafety_span: unsafety.span,
            });
        }

        if let Some(asyncness) = self.new_method.sig.asyncness {
            return Err(InjectableImplError::NewMethodAsync {
                asyncness_span: asyncness.span,
            });
        }

        if !self.new_method.sig.generics.params.is_empty() {
            return Err(InjectableImplError::NewMethodGeneric {
                generics_span: self.new_method.sig.generics.span(),
            });
        }
        Ok(())
    }

    #[cfg(not(tarpaulin_include))]
    pub fn expand(&self, no_doc_hidden: bool, is_async: bool)
        -> proc_macro2::TokenStream
    {
        let di_container_var = format_ident!("{}", DI_CONTAINER_VAR_NAME);
        let dependency_history_var = format_ident!("{}", DEPENDENCY_HISTORY_VAR_NAME);

        let maybe_doc_hidden = if no_doc_hidden {
            quote! {}
        } else {
            quote! { #[doc(hidden)] }
        };

        let maybe_prevent_circular_deps = if cfg!(feature = "prevent-circular") {
            Self::expand_prevent_circular_deps(&dependency_history_var)
        } else {
            quote! {}
        };

        let injectable_impl = if is_async {
            self.expand_async_impl(
                &maybe_doc_hidden,
                &di_container_var,
                &dependency_history_var,
                &maybe_prevent_circular_deps,
                &Self::create_get_dep_method_calls(&self.dependencies, true).unwrap(),
            )
        } else {
            self.expand_blocking_impl(
                &maybe_doc_hidden,
                &di_container_var,
                &dependency_history_var,
                &maybe_prevent_circular_deps,
                &Self::create_get_dep_method_calls(&self.dependencies, false).unwrap(),
            )
        };

        let original_impl = &self.original_impl;

        quote! {
            #original_impl

            #injectable_impl
        }
    }

    #[cfg(not(tarpaulin_include))]
    fn expand_prevent_circular_deps(
        dependency_history_var: &Ident,
    ) -> proc_macro2::TokenStream
    {
        quote! {
            use syrette::dependency_history::IDependencyHistory;

            if #dependency_history_var.contains::<Self>() {
                #dependency_history_var.push::<Self>();

                return Err(InjectableError::DetectedCircular {
                    dependency_history: #dependency_history_var
                });
            }

            #dependency_history_var.push::<Self>();
        }
    }

    #[cfg(not(tarpaulin_include))]
    fn expand_async_impl(
        &self,
        maybe_doc_hidden: &proc_macro2::TokenStream,
        di_container_var: &Ident,
        dependency_history_var: &Ident,
        maybe_prevent_circular_deps: &proc_macro2::TokenStream,
        get_dep_method_calls: &Vec<proc_macro2::TokenStream>,
    ) -> proc_macro2::TokenStream
    {
        let generics = &self.generics;
        let self_type = &self.self_type;

        quote! {
            #maybe_doc_hidden
            impl #generics syrette::interfaces::async_injectable::AsyncInjectable<
                syrette::di_container::asynchronous::AsyncDIContainer,
                syrette::dependency_history::DependencyHistory
            > for #self_type
            {
                fn resolve<'di_container, 'fut>(
                    #di_container_var: &'di_container std::sync::Arc<
                        syrette::di_container::asynchronous::AsyncDIContainer
                    >,
                    mut #dependency_history_var: syrette::dependency_history::DependencyHistory
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

                        use syrette::di_container::asynchronous::IAsyncDIContainer;
                        use syrette::errors::injectable::InjectableError;

                        let self_type_name = type_name::<#self_type>();

                        #maybe_prevent_circular_deps

                        Ok(syrette::ptr::TransientPtr::new(Self::new(
                            #(#get_dep_method_calls),*
                        )))
                    })
                }
            }
        }
    }

    #[cfg(not(tarpaulin_include))]
    fn expand_blocking_impl(
        &self,
        maybe_doc_hidden: &proc_macro2::TokenStream,
        di_container_var: &Ident,
        dependency_history_var: &Ident,
        maybe_prevent_circular_deps: &proc_macro2::TokenStream,
        get_dep_method_calls: &Vec<proc_macro2::TokenStream>,
    ) -> proc_macro2::TokenStream
    {
        let generics = &self.generics;
        let self_type = &self.self_type;

        quote! {
            #maybe_doc_hidden
            impl #generics syrette::interfaces::injectable::Injectable<
                syrette::di_container::blocking::DIContainer,
                syrette::dependency_history::DependencyHistory
            > for #self_type
            {
                fn resolve(
                    #di_container_var: &std::rc::Rc<
                        syrette::di_container::blocking::DIContainer
                    >,
                    mut #dependency_history_var: syrette::dependency_history::DependencyHistory
                ) -> Result<
                    syrette::ptr::TransientPtr<Self>,
                    syrette::errors::injectable::InjectableError>
                {
                    use std::any::type_name;

                    use syrette::di_container::blocking::IDIContainer;
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

    #[cfg(not(tarpaulin_include))]
    pub fn expand_dummy_blocking_impl(&self) -> proc_macro2::TokenStream
    {
        let generics = &self.generics;
        let self_type = &self.self_type;

        let di_container_var = format_ident!("{}", DI_CONTAINER_VAR_NAME);
        let dependency_history_var = format_ident!("{}", DEPENDENCY_HISTORY_VAR_NAME);

        quote! {
            impl #generics syrette::interfaces::injectable::Injectable<
                syrette::di_container::blocking::DIContainer,
                syrette::dependency_history::DependencyHistory
            > for #self_type
            {
                fn resolve(
                    #di_container_var: &std::rc::Rc<
                        syrette::di_container::blocking::DIContainer
                    >,
                    mut #dependency_history_var: syrette::dependency_history::DependencyHistory
                ) -> Result<
                    syrette::ptr::TransientPtr<Self>,
                    syrette::errors::injectable::InjectableError>
                {
                    unimplemented!();
                }
            }
        }
    }

    #[cfg(not(tarpaulin_include))]
    pub fn expand_dummy_async_impl(&self) -> proc_macro2::TokenStream
    {
        let generics = &self.generics;
        let self_type = &self.self_type;

        let di_container_var = format_ident!("{}", DI_CONTAINER_VAR_NAME);
        let dependency_history_var = format_ident!("{}", DEPENDENCY_HISTORY_VAR_NAME);

        quote! {
            impl #generics syrette::interfaces::async_injectable::AsyncInjectable<
                syrette::di_container::asynchronous::AsyncDIContainer,
                syrette::dependency_history::DependencyHistory
            > for #self_type
            {
                fn resolve<'di_container, 'fut>(
                    #di_container_var: &'di_container std::sync::Arc<
                        syrette::di_container::asynchronous::AsyncDIContainer
                    >,
                    mut #dependency_history_var: syrette::dependency_history::DependencyHistory
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
                    unimplemented!();
                }
            }
        }
    }

    fn create_get_dep_method_calls(
        dependencies: &[Dep],
        is_async: bool,
    ) -> Result<Vec<proc_macro2::TokenStream>, Box<dyn Error>>
    {
        dependencies
            .iter()
            .filter_map(|dependency| {
                match dependency.get_interface() {
                    Type::TraitObject(_) | Type::Path(_) => Some(()),
                    _ => None,
                }?;

                Some(Self::create_single_get_dep_method_call(
                    dependency, is_async,
                ))
            })
            .collect()
    }

    fn create_single_get_dep_method_call(
        dependency: &Dep,
        is_async: bool,
    ) -> Result<proc_macro2::TokenStream, Box<dyn Error>>
    {
        let dep_interface_str = match &dependency.get_interface() {
            Type::TraitObject(interface_trait) => {
                Ok(interface_trait.to_token_stream().to_string())
            }
            Type::Path(path_interface) => Ok(path_interface.path.to_string()),
            &_ => Err("Invalid type. Expected trait type or path type"),
        }?;

        let method_call = parse_str::<ExprMethodCall>(
            format!(
                "{}.get_bound::<{}>({}.clone(), {})",
                DI_CONTAINER_VAR_NAME,
                dep_interface_str,
                DEPENDENCY_HISTORY_VAR_NAME,
                dependency.get_name().as_ref().map_or_else(
                    || "None".to_string(),
                    |name| format!("Some(\"{}\")", name.value())
                )
            )
            .as_str(),
        )?;

        let ptr_name = dependency.get_ptr().to_string();

        let to_ptr =
            format_ident!("{}", camelcase_to_snakecase(&ptr_name.replace("Ptr", "")));

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

        Ok(quote! {
            #do_method_call
                .map_err(|err| #resolve_failed_error {
                    reason: Box::new(err),
                    affected: self_type_name
                })?
                .#to_ptr()
                .map_err(|err| InjectableError:: PrepareDependencyFailed {
                    reason: err,
                    dependency_name: #dep_interface_str
                })?
        })
    }

    fn build_dependencies(
        new_method: &ImplItemMethod,
    ) -> Result<Vec<Dep>, DependencyError>
    {
        let new_method_args = &new_method.sig.inputs;

        let dependencies_result: Result<Vec<_>, _> =
            new_method_args.iter().map(Dep::build).collect();

        let deps = dependencies_result?;

        Ok(deps)
    }

    // Removes argument attributes from a method, as they are not actually valid Rust.
    // Not doing this would cause a compilation error.
    fn remove_method_argument_attrs(method: &mut ImplItemMethod)
    {
        let method_args = &mut method.sig.inputs;

        for arg in method_args {
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
                    if &attr.path.to_string() == "syrette::named" {
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
    }
}

diagnostic_error_enum! {
pub enum InjectableImplError
{
    #[
        error("The 'injectable' attribute must be placed on a implementation"),
        span = err_span
    ]
    NotAImplementation
    {
        err_span: Span
    },

    #[
        error("The 'injectable' attribute cannot be placed on a trait implementation"),
        span = trait_path_span
    ]
    TraitImpl
    {
        trait_path_span: Span
    },

    #[error("Missing a 'new' method"), span = implementation_span]
    #[note("Required by the 'injectable' attribute macro")]
    MissingNewMethod {
        implementation_span: Span
    },

    #[
        error(concat!(
            "Invalid 'new' method return type. Expected it to be '{}'. ",
            "Found '{}'"
        ), expected, found),
        span = new_method_output_span
    ]
    InvalidNewMethodReturnType
    {
        new_method_output_span: Span,
        expected: String,
        found: String
    },

    #[error("'new' method is not allowed to be unsafe"), span = unsafety_span]
    #[note("Required by the 'injectable' attribute macro")]
    NewMethodUnsafe {
        unsafety_span: Span
    },

    #[error("'new' method is not allowed to be async"), span = asyncness_span]
    #[note("Required by the 'injectable' attribute macro")]
    NewMethodAsync {
        asyncness_span: Span
    },

    #[error("'new' method is not allowed to have generics"), span = generics_span]
    #[note("Required by the 'injectable' attribute macro")]
    NewMethodGeneric {
        generics_span: Span
    },

    #[error("Has a invalid dependency"), span = implementation_span]
    #[source(err)]
    ContainsAInvalidDependency {
        implementation_span: Span,
        err: DependencyError
    },
}
}

#[cfg(test)]
mod tests
{
    use std::sync::{Mutex, MutexGuard};

    use once_cell::sync::Lazy;
    use pretty_assertions::assert_eq;
    use proc_macro2::{Span, TokenStream};
    use syn::token::{Brace, Bracket, Colon, Paren, Pound};
    use syn::{
        parse2,
        AttrStyle,
        Attribute,
        Block,
        ImplItemMethod,
        LitStr,
        Pat,
        PatType,
        Visibility,
    };

    use super::*;
    use crate::injectable::dependency::MockIDependency;
    use crate::injectable::named_attr_input::NamedAttrInput;
    use crate::test_utils;

    static TEST_MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    // When a test panics, it will poison the Mutex. Since we don't actually
    // care about the state of the data we ignore that it is poisoned and grab
    // the lock regardless.  If you just do `let _lock = &TEST_MUTEX.lock().unwrap()`, one
    // test panicking will cause all other tests that try and acquire a lock on
    // that Mutex to also panic.
    fn get_lock(m: &'static Mutex<()>) -> MutexGuard<'static, ()>
    {
        match m.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        }
    }

    #[test]
    fn can_build_dependencies() -> Result<(), Box<dyn Error>>
    {
        let method = ImplItemMethod {
            attrs: vec![],
            vis: Visibility::Inherited,
            defaultness: None,
            sig: test_utils::create_signature(
                format_ident!("new"),
                vec![
                    (
                        test_utils::create_type(test_utils::create_path(&[
                            test_utils::create_path_segment(
                                format_ident!("TransientPtr"),
                                &[test_utils::create_type(test_utils::create_path(&[
                                    test_utils::create_path_segment(
                                        format_ident!("Foo"),
                                        &[],
                                    ),
                                ]))],
                            ),
                        ])),
                        vec![],
                    ),
                    (
                        test_utils::create_type(test_utils::create_path(&[
                            test_utils::create_path_segment(
                                format_ident!("FactoryPtr"),
                                &[test_utils::create_type(test_utils::create_path(&[
                                    test_utils::create_path_segment(
                                        format_ident!("BarFactory"),
                                        &[],
                                    ),
                                ]))],
                            ),
                        ])),
                        vec![],
                    ),
                ],
                test_utils::create_type(test_utils::create_path(&[
                    test_utils::create_path_segment(format_ident!("Self"), &[]),
                ])),
            ),
            block: Block {
                brace_token: Brace::default(),
                stmts: vec![],
            },
        };

        let _lock = get_lock(&TEST_MUTEX);

        let build_context = MockIDependency::build_context();

        build_context
            .expect()
            .returning(|_| Ok(MockIDependency::new()));

        let dependencies = InjectableImpl::<MockIDependency>::build_dependencies(&method)
            .expect("Expected Ok");

        assert_eq!(dependencies.len(), 2);

        Ok(())
    }

    #[test]
    fn can_build_named_dependencies()
    {
        let method = ImplItemMethod {
            attrs: vec![],
            vis: Visibility::Inherited,
            defaultness: None,
            sig: test_utils::create_signature(
                format_ident!("new"),
                vec![
                    (
                        test_utils::create_type(test_utils::create_path(&[
                            test_utils::create_path_segment(
                                format_ident!("TransientPtr"),
                                &[test_utils::create_type(test_utils::create_path(&[
                                    test_utils::create_path_segment(
                                        format_ident!("Foo"),
                                        &[],
                                    ),
                                ]))],
                            ),
                        ])),
                        vec![],
                    ),
                    (
                        test_utils::create_type(test_utils::create_path(&[
                            test_utils::create_path_segment(
                                format_ident!("FactoryPtr"),
                                &[test_utils::create_type(test_utils::create_path(&[
                                    test_utils::create_path_segment(
                                        format_ident!("BarFactory"),
                                        &[],
                                    ),
                                ]))],
                            ),
                        ])),
                        vec![Attribute {
                            pound_token: Pound::default(),
                            style: AttrStyle::Outer,
                            bracket_token: Bracket::default(),
                            path: test_utils::create_path(&[
                                test_utils::create_path_segment(
                                    format_ident!("named"),
                                    &[],
                                ),
                            ]),
                            tokens: NamedAttrInput {
                                paren: Paren::default(),
                                name: LitStr::new("awesome", Span::call_site()),
                            }
                            .to_token_stream(),
                        }],
                    ),
                ],
                test_utils::create_type(test_utils::create_path(&[
                    test_utils::create_path_segment(format_ident!("Self"), &[]),
                ])),
            ),
            block: Block {
                brace_token: Brace::default(),
                stmts: vec![],
            },
        };

        let _lock = get_lock(&TEST_MUTEX);

        let build_context = MockIDependency::build_context();

        build_context
            .expect()
            .returning(|_| Ok(MockIDependency::new()))
            .times(2);

        let dependencies = InjectableImpl::<MockIDependency>::build_dependencies(&method)
            .expect("Expected Ok");

        assert_eq!(dependencies.len(), 2);
    }

    #[test]
    fn can_remove_method_argument_attrs()
    {
        let first_arg_type = test_utils::create_type(test_utils::create_path(&[
            test_utils::create_path_segment(
                format_ident!("TransientPtr"),
                &[test_utils::create_type(test_utils::create_path(&[
                    test_utils::create_path_segment(format_ident!("Foo"), &[]),
                ]))],
            ),
        ]));

        let second_arg_type = test_utils::create_type(test_utils::create_path(&[
            test_utils::create_path_segment(
                format_ident!("FactoryPtr"),
                &[test_utils::create_type(test_utils::create_path(&[
                    test_utils::create_path_segment(format_ident!("BarFactory"), &[]),
                ]))],
            ),
        ]));

        let mut method = ImplItemMethod {
            attrs: vec![],
            vis: Visibility::Inherited,
            defaultness: None,
            sig: test_utils::create_signature(
                format_ident!("new"),
                vec![
                    (
                        first_arg_type.clone(),
                        vec![Attribute {
                            pound_token: Pound::default(),
                            style: AttrStyle::Outer,
                            bracket_token: Bracket::default(),
                            path: test_utils::create_path(&[
                                test_utils::create_path_segment(
                                    format_ident!("named"),
                                    &[],
                                ),
                            ]),
                            tokens: NamedAttrInput {
                                paren: Paren::default(),
                                name: LitStr::new("cool", Span::call_site()),
                            }
                            .to_token_stream(),
                        }],
                    ),
                    (
                        second_arg_type.clone(),
                        vec![Attribute {
                            pound_token: Pound::default(),
                            style: AttrStyle::Outer,
                            bracket_token: Bracket::default(),
                            path: test_utils::create_path(&[
                                test_utils::create_path_segment(
                                    format_ident!("named"),
                                    &[],
                                ),
                            ]),
                            tokens: NamedAttrInput {
                                paren: Paren::default(),
                                name: LitStr::new("awesome", Span::call_site()),
                            }
                            .to_token_stream(),
                        }],
                    ),
                ],
                test_utils::create_type(test_utils::create_path(&[
                    test_utils::create_path_segment(format_ident!("Self"), &[]),
                ])),
            ),
            block: Block {
                brace_token: Brace::default(),
                stmts: vec![],
            },
        };

        InjectableImpl::<MockIDependency>::remove_method_argument_attrs(&mut method);

        assert_eq!(
            method.sig.inputs.first().unwrap().clone(),
            FnArg::Typed(PatType {
                attrs: vec![],
                pat: Box::new(Pat::Verbatim(TokenStream::new())),
                colon_token: Colon::default(),
                ty: Box::new(first_arg_type),
            })
        );

        assert_eq!(
            method.sig.inputs.last().unwrap().clone(),
            FnArg::Typed(PatType {
                attrs: vec![],
                pat: Box::new(Pat::Verbatim(TokenStream::new())),
                colon_token: Colon::default(),
                ty: Box::new(second_arg_type),
            })
        );
    }

    #[test]
    fn can_create_single_get_dep_method_call() -> Result<(), Box<dyn Error>>
    {
        let mut mock_dependency = MockIDependency::new();

        mock_dependency
            .expect_get_interface()
            .return_const(test_utils::create_type(test_utils::create_path(&[
                test_utils::create_path_segment(format_ident!("Foo"), &[]),
            ])));

        mock_dependency.expect_get_name().return_const(None);

        mock_dependency
            .expect_get_ptr()
            .return_const(format_ident!("TransientPtr"));

        let di_container_var_ident = format_ident!("{}", DI_CONTAINER_VAR_NAME);
        let dep_history_var_ident = format_ident!("{}", DEPENDENCY_HISTORY_VAR_NAME);

        let output =
            InjectableImpl::<MockIDependency>::create_single_get_dep_method_call(
                &mock_dependency,
                false,
            )?;

        assert_eq!(
            parse2::<ExprMethodCall>(output)?,
            parse2::<ExprMethodCall>(quote! {
                #di_container_var_ident
                    .get_bound::<Foo>(#dep_history_var_ident.clone(), None)
                    .map_err(|err| InjectableError::ResolveFailed {
                        reason: Box::new(err),
                        affected: self_type_name
                    })?
                    .transient()
                    .unwrap()

            })?
        );

        Ok(())
    }

    #[test]
    fn can_create_single_get_dep_method_call_with_name() -> Result<(), Box<dyn Error>>
    {
        let mut mock_dependency = MockIDependency::new();

        mock_dependency
            .expect_get_interface()
            .return_const(test_utils::create_type(test_utils::create_path(&[
                test_utils::create_path_segment(format_ident!("Foo"), &[]),
            ])));

        mock_dependency
            .expect_get_name()
            .return_const(Some(LitStr::new("special", Span::call_site())));

        mock_dependency
            .expect_get_ptr()
            .return_const(format_ident!("TransientPtr"));

        let di_container_var_ident = format_ident!("{}", DI_CONTAINER_VAR_NAME);
        let dep_history_var_ident = format_ident!("{}", DEPENDENCY_HISTORY_VAR_NAME);

        let output =
            InjectableImpl::<MockIDependency>::create_single_get_dep_method_call(
                &mock_dependency,
                false,
            )?;

        assert_eq!(
            parse2::<ExprMethodCall>(output)?,
            parse2::<ExprMethodCall>(quote! {
                #di_container_var_ident
                    .get_bound::<Foo>(#dep_history_var_ident.clone(), Some("special"))
                    .map_err(|err| InjectableError::ResolveFailed {
                        reason: Box::new(err),
                        affected: self_type_name
                    })?
                    .transient()
                    .unwrap()

            })?
        );

        Ok(())
    }

    #[test]
    fn can_create_single_get_dep_method_call_async() -> Result<(), Box<dyn Error>>
    {
        let mut mock_dependency = MockIDependency::new();

        mock_dependency
            .expect_get_interface()
            .return_const(test_utils::create_type(test_utils::create_path(&[
                test_utils::create_path_segment(format_ident!("Foo"), &[]),
            ])));

        mock_dependency.expect_get_name().return_const(None);

        mock_dependency
            .expect_get_ptr()
            .return_const(format_ident!("TransientPtr"));

        let di_container_var_ident = format_ident!("{}", DI_CONTAINER_VAR_NAME);
        let dep_history_var_ident = format_ident!("{}", DEPENDENCY_HISTORY_VAR_NAME);

        let output =
            InjectableImpl::<MockIDependency>::create_single_get_dep_method_call(
                &mock_dependency,
                true,
            )?;

        assert_eq!(
            parse2::<ExprMethodCall>(output)?,
            parse2::<ExprMethodCall>(quote! {
                #di_container_var_ident
                    .get_bound::<Foo>(#dep_history_var_ident.clone(), None)
                    .await
                    .map_err(|err| InjectableError::AsyncResolveFailed {
                        reason: Box::new(err),
                        affected: self_type_name
                    })?
                    .transient()
                    .unwrap()

            })?
        );

        Ok(())
    }

    #[test]
    fn can_create_single_get_dep_method_call_async_with_name(
    ) -> Result<(), Box<dyn Error>>
    {
        let mut mock_dependency = MockIDependency::new();

        mock_dependency
            .expect_get_interface()
            .return_const(test_utils::create_type(test_utils::create_path(&[
                test_utils::create_path_segment(format_ident!("Foo"), &[]),
            ])));

        mock_dependency
            .expect_get_name()
            .return_const(Some(LitStr::new("foobar", Span::call_site())));

        mock_dependency
            .expect_get_ptr()
            .return_const(format_ident!("TransientPtr"));

        let di_container_var_ident = format_ident!("{}", DI_CONTAINER_VAR_NAME);
        let dep_history_var_ident = format_ident!("{}", DEPENDENCY_HISTORY_VAR_NAME);

        let output =
            InjectableImpl::<MockIDependency>::create_single_get_dep_method_call(
                &mock_dependency,
                true,
            )?;

        assert_eq!(
            parse2::<ExprMethodCall>(output)?,
            parse2::<ExprMethodCall>(quote! {
                #di_container_var_ident
                    .get_bound::<Foo>(#dep_history_var_ident.clone(), Some("foobar"))
                    .await
                    .map_err(|err| InjectableError::AsyncResolveFailed {
                        reason: Box::new(err),
                        affected: self_type_name
                    })?
                    .transient()
                    .unwrap()

            })?
        );

        Ok(())
    }
}
