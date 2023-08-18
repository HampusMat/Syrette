use proc_macro2::{Ident, Span};
use syn::spanned::Spanned;
use syn::{parse2, FnArg, GenericArgument, LitStr, PathArguments, Type};

use crate::injectable::named_attr_input::NamedAttrInput;
use crate::util::error::diagnostic_error_enum;
use crate::util::syn_path::SynPathExt;

/// Interface for a dependency of a `Injectable`.
#[cfg_attr(test, mockall::automock)]
pub trait IDependency: Sized
{
    /// Build a new `Dependency` from a argument in a constructor method.
    fn build(ctor_method_arg: &FnArg) -> Result<Self, DependencyError>;

    /// Returns the interface type.
    fn get_interface(&self) -> &Type;

    /// Returns the pointer type identity.
    fn get_ptr(&self) -> &Ident;

    /// Returns optional name of the dependency.
    fn get_name(&self) -> &Option<LitStr>;
}

/// Representation of a dependency of a injectable type.
///
/// Found as a argument in the constructor method of a `Injectable`.
#[derive(Debug, PartialEq, Eq)]
pub struct Dependency
{
    interface: Type,
    ptr: Ident,
    name: Option<LitStr>,
}

impl IDependency for Dependency
{
    fn build(ctor_method_arg: &FnArg) -> Result<Self, DependencyError>
    {
        let typed_ctor_method_arg = match ctor_method_arg {
            FnArg::Typed(typed_arg) => Ok(typed_arg),
            FnArg::Receiver(receiver_arg) => Err(DependencyError::UnexpectedSelf {
                self_token_span: receiver_arg.self_token.span,
            }),
        }?;

        let dependency_type_path = match typed_ctor_method_arg.ty.as_ref() {
            Type::Path(arg_type_path) => Ok(arg_type_path),
            Type::Reference(ref_type_path) => match ref_type_path.elem.as_ref() {
                Type::Path(arg_type_path) => Ok(arg_type_path),
                other_type => Err(DependencyError::InvalidType {
                    type_span: other_type.span(),
                }),
            },
            other_type => Err(DependencyError::InvalidType {
                type_span: other_type.span(),
            }),
        }?;

        let ptr_path_segment = dependency_type_path.path.segments.last().map_or_else(
            || {
                Err(DependencyError::MissingType {
                    arg_span: typed_ctor_method_arg.span(),
                })
            },
            Ok,
        )?;

        let ptr_ident = ptr_path_segment.ident.clone();

        let ptr_generic_args = match ptr_path_segment.arguments.clone() {
            PathArguments::AngleBracketed(generic_args) => Ok(generic_args),
            _ => Err(DependencyError::DependencyTypeMissingGenerics {
                ptr_ident_span: ptr_ident.span(),
            }),
        }?
        .args;

        let interface =
            if let Some(GenericArgument::Type(interface)) = ptr_generic_args.first() {
                Ok(interface.clone())
            } else {
                Err(DependencyError::DependencyTypeMissingGenerics {
                    ptr_ident_span: ptr_ident.span(),
                })
            }?;

        let arg_attrs = &typed_ctor_method_arg.attrs;

        let opt_named_attr = arg_attrs.iter().find(|attr| {
            attr.path.get_ident().map_or_else(
                || false,
                |attr_ident| attr_ident.to_string().as_str() == "named",
            ) || &attr.path.to_string() == "syrette::named"
        });

        let opt_named_attr_tokens = opt_named_attr.map(|attr| &attr.tokens);

        let opt_named_attr_input =
            if let Some(named_attr_tokens) = opt_named_attr_tokens {
                Some(parse2::<NamedAttrInput>(named_attr_tokens.clone()).map_err(
                    |err| DependencyError::InvalidNamedAttrInput {
                        arg_span: typed_ctor_method_arg.span(),
                        err,
                    },
                )?)
            } else {
                None
            };

        Ok(Self {
            interface,
            ptr: ptr_ident,
            name: opt_named_attr_input.map(|named_attr_input| named_attr_input.name),
        })
    }

    fn get_interface(&self) -> &Type
    {
        &self.interface
    }

    fn get_ptr(&self) -> &Ident
    {
        &self.ptr
    }

    fn get_name(&self) -> &Option<LitStr>
    {
        &self.name
    }
}

diagnostic_error_enum! {
pub enum DependencyError
{
    #[error("Unexpected 'self' parameter"), span = self_token_span]
    #[help("Remove the 'self' parameter"), span = self_token_span]
    UnexpectedSelf {
        self_token_span: Span
    },

    #[
        error("Dependency type must either be a path or a path reference"),
        span = type_span
    ]
    InvalidType {
        type_span: Span
    },

    #[error("Dependency is missing a type"), span = arg_span]
    MissingType {
        arg_span: Span
    },

    #[
        error("Expected dependency type to take generic parameters"),
        span = ptr_ident_span
    ]
    DependencyTypeMissingGenerics {
        ptr_ident_span: Span
    },

    #[error("Dependency has a 'named' attribute given invalid input"), span = arg_span]
    #[source(err)]
    InvalidNamedAttrInput {
        arg_span: Span,
        err: syn::Error
    },
}
}

#[cfg(test)]
mod tests
{
    use proc_macro2::{Span, TokenStream};
    use quote::{format_ident, quote};
    use syn::punctuated::Punctuated;
    use syn::token::{And, Bang, Bracket, Colon, Paren, Pound, SelfValue};
    use syn::{
        AttrStyle,
        Attribute,
        Pat,
        PatType,
        PathSegment,
        Receiver,
        TypeNever,
        TypeReference,
        TypeTuple,
    };

    use super::*;
    use crate::test_utils;

    #[test]
    fn can_build_dependency()
    {
        assert!(matches!(
            Dependency::build(&FnArg::Typed(PatType {
                attrs: vec![],
                pat: Box::new(Pat::Verbatim(TokenStream::default())),
                colon_token: Colon::default(),
                ty: Box::new(test_utils::create_type(test_utils::create_path(&[
                    test_utils::create_path_segment(
                        format_ident!("TransientPtr"),
                        &[test_utils::create_type(test_utils::create_path(&[
                            test_utils::create_path_segment(format_ident!("Foo"), &[])
                        ]))]
                    ),
                ])))
            })),
            Ok(dependency) if dependency == Dependency {
                interface: test_utils::create_type(test_utils::create_path(&[
                    PathSegment::from(format_ident!("Foo"))
                ])),
                ptr: format_ident!("TransientPtr"),
                name: None
            }
        ));

        assert!(matches!(
            Dependency::build(&FnArg::Typed(PatType {
                attrs: vec![],
                pat: Box::new(Pat::Verbatim(TokenStream::default())),
                colon_token: Colon::default(),
                ty: Box::new(test_utils::create_type(test_utils::create_path(&[
                    test_utils::create_path_segment(format_ident!("syrette"), &[]),
                    test_utils::create_path_segment(format_ident!("ptr"), &[]),
                    test_utils::create_path_segment(
                        format_ident!("SingletonPtr"),
                        &[test_utils::create_type(test_utils::create_path(&[
                            test_utils::create_path_segment(format_ident!("Bar"), &[])
                        ]))]
                    ),
                ])))
            })),
            Ok(dependency) if dependency == Dependency {
                interface: test_utils::create_type(test_utils::create_path(&[
                    PathSegment::from(format_ident!("Bar"))
                ])),
                ptr: format_ident!("SingletonPtr"),
                name: None
            }
        ));
    }

    #[test]
    fn can_build_dependency_with_name()
    {
        assert!(matches!(
            Dependency::build(&FnArg::Typed(PatType {
                attrs: vec![Attribute {
                    pound_token: Pound::default(),
                    style: AttrStyle::Outer,
                    bracket_token: Bracket::default(),
                    path: test_utils::create_path(&[test_utils::create_path_segment(
                        format_ident!("named"),
                        &[]
                    )]),
                    tokens: quote! { ("cool") }
                }],
                pat: Box::new(Pat::Verbatim(TokenStream::default())),
                colon_token: Colon::default(),
                ty: Box::new(test_utils::create_type(test_utils::create_path(&[
                    test_utils::create_path_segment(
                        format_ident!("TransientPtr"),
                        &[test_utils::create_type(test_utils::create_path(&[
                            test_utils::create_path_segment(format_ident!("Foo"), &[])
                        ]))]
                    ),
                ])))
            })),
            Ok(dependency) if dependency == Dependency {
                interface: test_utils::create_type(test_utils::create_path(&[
                    PathSegment::from(format_ident!("Foo"))
                ])),
                ptr: format_ident!("TransientPtr"),
                name: Some(LitStr::new("cool", Span::call_site()))
            }
        ));

        assert!(matches!(
            Dependency::build(&FnArg::Typed(PatType {
                attrs: vec![Attribute {
                    pound_token: Pound::default(),
                    style: AttrStyle::Outer,
                    bracket_token: Bracket::default(),
                    path: test_utils::create_path(&[test_utils::create_path_segment(
                        format_ident!("named"),
                        &[]
                    )]),
                    tokens: quote! { ("awesome") }
                }],
                pat: Box::new(Pat::Verbatim(TokenStream::default())),
                colon_token: Colon::default(),
                ty: Box::new(test_utils::create_type(test_utils::create_path(&[
                    test_utils::create_path_segment(format_ident!("syrette"), &[]),
                    test_utils::create_path_segment(format_ident!("ptr"), &[]),
                    test_utils::create_path_segment(
                        format_ident!("FactoryPtr"),
                        &[test_utils::create_type(test_utils::create_path(&[
                            test_utils::create_path_segment(format_ident!("Bar"), &[])
                        ]))]
                    ),
                ])))
            })),
            Ok(dependency) if dependency == Dependency {
                interface: test_utils::create_type(test_utils::create_path(&[
                    PathSegment::from(format_ident!("Bar"))
                ])),
                ptr: format_ident!("FactoryPtr"),
                name: Some(LitStr::new("awesome", Span::call_site()))
            }
        ));
    }

    #[test]
    fn cannot_build_dependency_with_receiver_arg()
    {
        assert!(Dependency::build(&FnArg::Receiver(Receiver {
            attrs: vec![],
            reference: None,
            mutability: None,
            self_token: SelfValue::default()
        }))
        .is_err());
    }

    #[test]
    fn cannot_build_dependency_with_type_not_path()
    {
        assert!(Dependency::build(&FnArg::Typed(PatType {
            attrs: vec![],
            pat: Box::new(Pat::Verbatim(TokenStream::default())),
            colon_token: Colon::default(),
            ty: Box::new(Type::Tuple(TypeTuple {
                paren_token: Paren::default(),
                elems: Punctuated::from_iter(vec![test_utils::create_type(
                    test_utils::create_path(&[test_utils::create_path_segment(
                        format_ident!("EvilType"),
                        &[]
                    )])
                )])
            }))
        }))
        .is_err());

        assert!(Dependency::build(&FnArg::Typed(PatType {
            attrs: vec![],
            pat: Box::new(Pat::Verbatim(TokenStream::default())),
            colon_token: Colon::default(),
            ty: Box::new(Type::Reference(TypeReference {
                and_token: And::default(),
                lifetime: None,
                mutability: None,
                elem: Box::new(Type::Never(TypeNever {
                    bang_token: Bang::default()
                }))
            }))
        }))
        .is_err());
    }

    #[test]
    fn cannot_build_dependency_without_generics_args()
    {
        assert!(Dependency::build(&FnArg::Typed(PatType {
            attrs: vec![],
            pat: Box::new(Pat::Verbatim(TokenStream::default())),
            colon_token: Colon::default(),
            ty: Box::new(test_utils::create_type(test_utils::create_path(&[
                test_utils::create_path_segment(format_ident!("TransientPtr"), &[]),
            ])))
        }))
        .is_err());
    }
}
