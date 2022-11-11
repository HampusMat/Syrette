use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Token, TypePath};

use crate::macro_flag::MacroFlag;
use crate::util::iterator_ext::IteratorExt;

pub const INJECTABLE_MACRO_FLAGS: &[&str] =
    &["no_doc_hidden", "async", "no_declare_concrete_interface"];

pub struct InjectableMacroArgs
{
    pub interface: Option<TypePath>,
    pub flags: Punctuated<MacroFlag, Token![,]>,
}

impl Parse for InjectableMacroArgs
{
    fn parse(input: ParseStream) -> syn::Result<Self>
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

        for flag in &flags {
            let flag_str = flag.flag.to_string();

            if !INJECTABLE_MACRO_FLAGS.contains(&flag_str.as_str()) {
                return Err(input.error(format!(
                    "Unknown flag '{}'. Expected one of [ {} ]",
                    flag_str,
                    INJECTABLE_MACRO_FLAGS.join(",")
                )));
            }
        }

        let flag_names = flags
            .iter()
            .map(|flag| flag.flag.to_string())
            .collect::<Vec<_>>();

        if let Some(dupe_flag_name) = flag_names.iter().find_duplicate() {
            return Err(input.error(format!("Duplicate flag '{dupe_flag_name}'")));
        }

        Ok(Self { interface, flags })
    }
}

#[cfg(test)]
mod tests
{
    use std::error::Error;

    use proc_macro2::Span;
    use quote::{format_ident, quote};
    use syn::{parse2, LitBool};

    use super::*;
    use crate::test_utils;

    #[test]
    fn can_parse_with_only_interface() -> Result<(), Box<dyn Error>>
    {
        let input_args = quote! {
            IFoo
        };

        let injectable_macro_args = parse2::<InjectableMacroArgs>(input_args)?;

        assert!(matches!(injectable_macro_args.interface, Some(interface)
            if interface == TypePath {
                qself: None,
                path: test_utils::create_path(&[
                    test_utils::create_path_segment(format_ident!("IFoo"), &[])
                ])
            }
        ));

        assert!(injectable_macro_args.flags.is_empty());

        Ok(())
    }

    #[test]
    fn can_parse_with_nothing() -> Result<(), Box<dyn Error>>
    {
        let input_args = quote! {};

        let injectable_macro_args = parse2::<InjectableMacroArgs>(input_args)?;

        assert!(matches!(injectable_macro_args.interface, None));

        assert!(injectable_macro_args.flags.is_empty());

        Ok(())
    }

    #[test]
    fn can_parse_with_interface_and_flags() -> Result<(), Box<dyn Error>>
    {
        let input_args = quote! {
            IFoo, no_doc_hidden = true, async = false
        };

        let injectable_macro_args = parse2::<InjectableMacroArgs>(input_args)?;

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
            Punctuated::from_iter(vec![
                MacroFlag {
                    flag: format_ident!("no_doc_hidden"),
                    is_on: LitBool::new(true, Span::call_site())
                },
                MacroFlag {
                    flag: format_ident!("async"),
                    is_on: LitBool::new(false, Span::call_site())
                }
            ])
        );

        Ok(())
    }

    #[test]
    fn can_parse_with_flags_only() -> Result<(), Box<dyn Error>>
    {
        let input_args = quote! {
            async = false, no_declare_concrete_interface = false
        };

        let injectable_macro_args = parse2::<InjectableMacroArgs>(input_args)?;

        assert!(matches!(injectable_macro_args.interface, None));

        assert_eq!(
            injectable_macro_args.flags,
            Punctuated::from_iter(vec![
                MacroFlag {
                    flag: format_ident!("async"),
                    is_on: LitBool::new(false, Span::call_site())
                },
                MacroFlag {
                    flag: format_ident!("no_declare_concrete_interface"),
                    is_on: LitBool::new(false, Span::call_site())
                }
            ])
        );

        Ok(())
    }

    #[test]
    fn cannot_parse_with_invalid_flag()
    {
        let input_args = quote! {
            IFoo, haha = true, async = false
        };

        assert!(matches!(parse2::<InjectableMacroArgs>(input_args), Err(_)));
    }

    #[test]
    fn cannot_parse_with_duplicate_flag()
    {
        assert!(matches!(
            parse2::<InjectableMacroArgs>(quote! {
                IFoo, async = false, no_doc_hidden = true, async = false
            }),
            Err(_)
        ));

        assert!(matches!(
            parse2::<InjectableMacroArgs>(quote! {
                IFoo, async = true , no_doc_hidden = true, async = false
            }),
            Err(_)
        ));
    }
}
