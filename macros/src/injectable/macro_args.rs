use proc_macro2::Span;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Ident, Token, TypePath};

use crate::macro_flag::MacroFlag;
use crate::util::error::diagnostic_error_enum;
use crate::util::iterator_ext::IteratorExt;

pub const INJECTABLE_MACRO_FLAGS: &[&str] = &[
    "no_doc_hidden",
    "async",
    "no_declare_concrete_interface",
    "constructor",
];

pub struct InjectableMacroArgs
{
    pub interface: Option<TypePath>,
    pub flags: Punctuated<MacroFlag, Token![,]>,
}

impl InjectableMacroArgs
{
    pub fn check_flags(&self) -> Result<(), InjectableMacroArgsError>
    {
        for flag in &self.flags {
            if !INJECTABLE_MACRO_FLAGS.contains(&flag.name().to_string().as_str()) {
                return Err(InjectableMacroArgsError::UnknownFlag {
                    flag_ident: flag.name().clone(),
                });
            }
        }

        if let Some((dupe_flag_first, dupe_flag_second)) =
            self.flags.iter().find_duplicate()
        {
            return Err(InjectableMacroArgsError::DuplicateFlag {
                first_flag_ident: dupe_flag_first.name().clone(),
                last_flag_span: dupe_flag_second.name().span(),
            });
        }

        Ok(())
    }
}

impl Parse for InjectableMacroArgs
{
    fn parse(input: ParseStream) -> Result<Self, syn::Error>
    {
        let input_fork = input.fork();

        let mut interface = None;

        if input_fork.parse::<MacroFlag>().is_err() {
            // Input doesn't begin with flags

            interface = input.parse::<TypePath>().ok();

            if interface.is_some() {
                let comma_input_lookahead = input.lookahead1();

                if !comma_input_lookahead.peek(Token![,]) {
                    return Ok(Self {
                        interface,
                        flags: Punctuated::new(),
                    });
                }

                input.parse::<Token![,]>()?;
            }

            if input.is_empty() {
                return Ok(Self {
                    interface,
                    flags: Punctuated::new(),
                });
            }
        }

        let flags = Punctuated::<MacroFlag, Token![,]>::parse_terminated(input)?;

        Ok(Self { interface, flags })
    }
}

diagnostic_error_enum! {
pub enum InjectableMacroArgsError
{
    #[error("Unknown flag '{flag_ident}'"), span = flag_ident.span()]
    #[
        help("Expected one of: {}", INJECTABLE_MACRO_FLAGS.join(", ")),
        span = flag_ident.span()
    ]
    UnknownFlag
    {
        flag_ident: Ident
    },

    #[error("Duplicate flag '{first_flag_ident}'"), span = first_flag_ident.span()]
    #[note("Previously mentioned here"), span = last_flag_span]
    DuplicateFlag
    {
        first_flag_ident: Ident,
        last_flag_span: Span
    },
}
}

#[cfg(test)]
mod tests
{
    use proc_macro2::Span;
    use quote::{format_ident, quote};
    use syn::{parse2, Lit, LitBool};

    use super::*;
    use crate::macro_flag::MacroFlagValue;
    use crate::test_utils;

    #[test]
    fn can_parse_with_only_interface()
    {
        let input_args = quote! {
            IFoo
        };

        let injectable_macro_args = parse2::<InjectableMacroArgs>(input_args).unwrap();

        assert!(matches!(injectable_macro_args.interface, Some(interface)
            if interface == TypePath {
                qself: None,
                path: test_utils::create_path(&[
                    test_utils::create_path_segment(format_ident!("IFoo"), &[])
                ])
            }
        ));

        assert!(injectable_macro_args.flags.is_empty());
    }

    #[test]
    fn can_parse_with_nothing()
    {
        let input_args = quote! {};

        let injectable_macro_args = parse2::<InjectableMacroArgs>(input_args).unwrap();

        assert!(injectable_macro_args.interface.is_none());
        assert!(injectable_macro_args.flags.is_empty());
    }

    #[test]
    fn can_parse_with_interface_and_flags()
    {
        let input_args = quote! {
            IFoo, no_doc_hidden = true, async = false
        };

        let injectable_macro_args = parse2::<InjectableMacroArgs>(input_args).unwrap();

        assert!(matches!(injectable_macro_args.interface, Some(interface)
            if interface == TypePath {
                qself: None,
                path: test_utils::create_path(&[
                    test_utils::create_path_segment(format_ident!("IFoo"), &[])
                ])
            }
        ));

        assert_eq!(
            injectable_macro_args.flags,
            Punctuated::from_iter([
                MacroFlag {
                    name: format_ident!("no_doc_hidden"),
                    value: MacroFlagValue::Literal(Lit::Bool(LitBool::new(
                        true,
                        Span::call_site()
                    )))
                },
                MacroFlag {
                    name: format_ident!("async"),
                    value: MacroFlagValue::Literal(Lit::Bool(LitBool::new(
                        false,
                        Span::call_site()
                    )))
                }
            ])
        );
    }

    #[test]
    fn can_parse_with_flags_only()
    {
        let input_args = quote! {
            async = false, no_declare_concrete_interface = false
        };

        let injectable_macro_args = parse2::<InjectableMacroArgs>(input_args).unwrap();

        assert!(injectable_macro_args.interface.is_none());

        assert_eq!(
            injectable_macro_args.flags,
            Punctuated::from_iter([
                MacroFlag {
                    name: format_ident!("async"),
                    value: MacroFlagValue::Literal(Lit::Bool(LitBool::new(
                        false,
                        Span::call_site()
                    )))
                },
                MacroFlag {
                    name: format_ident!("no_declare_concrete_interface"),
                    value: MacroFlagValue::Literal(Lit::Bool(LitBool::new(
                        false,
                        Span::call_site()
                    )))
                }
            ])
        );
    }

    #[test]
    fn can_parse_with_unknown_flag()
    {
        let input_args = quote! {
            IFoo, haha = true, async = false
        };

        assert!(parse2::<InjectableMacroArgs>(input_args).is_ok());
    }

    #[test]
    fn can_parse_with_duplicate_flag()
    {
        assert!(parse2::<InjectableMacroArgs>(quote! {
            IFoo, async = false, no_doc_hidden = true, async = false
        })
        .is_ok());

        assert!(parse2::<InjectableMacroArgs>(quote! {
            IFoo, async = true , no_doc_hidden = true, async = false
        })
        .is_ok());
    }

    #[test]
    fn check_flags_fail_with_unknown_flag()
    {
        let input_args = quote! {
            IFoo, haha = true, async = false
        };

        let injectable_macro_args = parse2::<InjectableMacroArgs>(input_args).unwrap();

        assert!(injectable_macro_args.check_flags().is_err());
    }

    #[test]
    fn check_flags_fail_with_duplicate_flag()
    {
        let macro_args = parse2::<InjectableMacroArgs>(quote! {
            IFoo, async = false, no_doc_hidden = true, async = false
        })
        .unwrap();

        assert!(macro_args.check_flags().is_err());

        let macro_args_two = parse2::<InjectableMacroArgs>(quote! {
            IFoo, async = true , no_doc_hidden = true, async = false
        })
        .unwrap();

        assert!(macro_args_two.check_flags().is_err());
    }
}
