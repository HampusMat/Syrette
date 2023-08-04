use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Token, TypePath};

use crate::macro_flag::MacroFlag;
use crate::util::iterator_ext::IteratorExt;

pub const DECLARE_INTERFACE_FLAGS: &[&str] = &["threadsafe_sharable"];

pub struct DeclareInterfaceArgs
{
    pub implementation: TypePath,
    pub interface: TypePath,
    pub flags: Punctuated<MacroFlag, Token![,]>,
}

impl Parse for DeclareInterfaceArgs
{
    fn parse(input: ParseStream) -> syn::Result<Self>
    {
        let implementation: TypePath = input.parse()?;

        input.parse::<Token![->]>()?;

        let interface: TypePath = input.parse()?;

        let flags = if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;

            let flags = Punctuated::<MacroFlag, Token![,]>::parse_terminated(input)?;

            for flag in &flags {
                let flag_name = flag.name().to_string();

                if !DECLARE_INTERFACE_FLAGS.contains(&flag_name.as_str()) {
                    return Err(input.error(format!(
                        "Unknown flag '{flag_name}'. Expected one of [ {} ]",
                        DECLARE_INTERFACE_FLAGS.join(",")
                    )));
                }
            }

            if let Some((dupe_flag, _)) = flags.iter().find_duplicate() {
                return Err(input.error(format!("Duplicate flag '{}'", dupe_flag.name())));
            }

            flags
        } else {
            Punctuated::new()
        };

        Ok(Self {
            implementation,
            interface,
            flags,
        })
    }
}

#[cfg(test)]
mod tests
{
    use std::error::Error;

    use proc_macro2::Span;
    use quote::{format_ident, quote};
    use syn::{parse2, Lit, LitBool};

    use super::*;
    use crate::macro_flag::MacroFlagValue;
    use crate::test_utils;

    #[test]
    fn can_parse_with_no_flags() -> Result<(), Box<dyn Error>>
    {
        let input_args = quote! {
            Foo -> IFoo
        };

        let decl_interface_args = parse2::<DeclareInterfaceArgs>(input_args)?;

        assert_eq!(
            decl_interface_args.implementation,
            TypePath {
                qself: None,
                path: test_utils::create_path(&[test_utils::create_path_segment(
                    format_ident!("Foo"),
                    &[]
                )])
            }
        );

        assert_eq!(
            decl_interface_args.interface,
            TypePath {
                qself: None,
                path: test_utils::create_path(&[test_utils::create_path_segment(
                    format_ident!("IFoo"),
                    &[]
                )])
            }
        );

        assert!(decl_interface_args.flags.is_empty());

        Ok(())
    }

    #[test]
    fn can_parse_with_flags() -> Result<(), Box<dyn Error>>
    {
        let input_args = quote! {
            Foobar -> IFoobar, threadsafe_sharable = true
        };

        let decl_interface_args = parse2::<DeclareInterfaceArgs>(input_args)?;

        assert_eq!(
            decl_interface_args.implementation,
            TypePath {
                qself: None,
                path: test_utils::create_path(&[test_utils::create_path_segment(
                    format_ident!("Foobar"),
                    &[]
                )])
            }
        );

        assert_eq!(
            decl_interface_args.interface,
            TypePath {
                qself: None,
                path: test_utils::create_path(&[test_utils::create_path_segment(
                    format_ident!("IFoobar"),
                    &[]
                )])
            }
        );

        assert_eq!(
            decl_interface_args.flags,
            Punctuated::from_iter(vec![MacroFlag {
                name: format_ident!("threadsafe_sharable"),
                value: MacroFlagValue::Literal(Lit::Bool(LitBool::new(
                    true,
                    Span::call_site()
                )))
            }])
        );

        Ok(())
    }

    #[test]
    fn cannot_parse_with_invalid_flag()
    {
        let input_args = quote! {
            Foobar -> IFoobar, xyz = false, threadsafe_sharable = true
        };

        assert!(parse2::<DeclareInterfaceArgs>(input_args).is_err());
    }

    #[test]
    fn cannot_parse_with_duplicate_flag()
    {
        assert!(
            // Formatting is weird without this comment
            parse2::<DeclareInterfaceArgs>(quote! {
                Foobar -> IFoobar, threadsafe_sharable = true, threadsafe_sharable = true
            })
            .is_err()
        );

        assert!(
            // Formatting is weird without this comment
            parse2::<DeclareInterfaceArgs>(quote! {
                Foobar -> IFoobar, threadsafe_sharable = true, threadsafe_sharable = false
            })
            .is_err()
        );
    }
}
