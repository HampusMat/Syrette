use syn::parse::Parse;
use syn::punctuated::Punctuated;
use syn::Token;

use crate::macro_flag::MacroFlag;
use crate::util::iterator_ext::IteratorExt;

pub const FACTORY_MACRO_FLAGS: &[&str] = &["threadsafe", "async"];

pub struct FactoryMacroArgs
{
    pub flags: Punctuated<MacroFlag, Token![,]>,
}

impl Parse for FactoryMacroArgs
{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self>
    {
        let flags = Punctuated::<MacroFlag, Token![,]>::parse_terminated(input)?;

        for flag in &flags {
            let flag_str = flag.flag.to_string();

            if !FACTORY_MACRO_FLAGS.contains(&flag_str.as_str()) {
                return Err(input.error(format!(
                    "Unknown flag '{}'. Expected one of [ {} ]",
                    flag_str,
                    FACTORY_MACRO_FLAGS.join(",")
                )));
            }
        }

        let flag_names = flags
            .iter()
            .map(|flag| flag.flag.to_string())
            .collect::<Vec<_>>();

        if let Some((dupe_flag_name, _)) = flag_names.iter().find_duplicate() {
            return Err(input.error(format!("Duplicate flag '{dupe_flag_name}'")));
        }

        Ok(Self { flags })
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

    #[test]
    fn can_parse_with_single_flag() -> Result<(), Box<dyn Error>>
    {
        let input_args = quote! {
            async = true
        };

        let factory_macro_args = parse2::<FactoryMacroArgs>(input_args)?;

        assert_eq!(
            factory_macro_args.flags,
            Punctuated::from_iter(vec![MacroFlag {
                flag: format_ident!("async"),
                is_on: LitBool::new(true, Span::call_site())
            }])
        );

        Ok(())
    }

    #[test]
    fn can_parse_with_multiple_flags() -> Result<(), Box<dyn Error>>
    {
        let input_args = quote! {
            async = true, threadsafe = false
        };

        let factory_macro_args = parse2::<FactoryMacroArgs>(input_args)?;

        assert_eq!(
            factory_macro_args.flags,
            Punctuated::from_iter(vec![
                MacroFlag {
                    flag: format_ident!("async"),
                    is_on: LitBool::new(true, Span::call_site())
                },
                MacroFlag {
                    flag: format_ident!("threadsafe"),
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
            async = true, threadsafe = false, foo = true
        };

        assert!(matches!(parse2::<FactoryMacroArgs>(input_args), Err(_)));
    }

    #[test]
    fn cannot_parse_with_duplicate_flag()
    {
        assert!(matches!(
            parse2::<FactoryMacroArgs>(quote! {
                async = true, threadsafe = false, async = true
            }),
            Err(_)
        ));

        assert!(matches!(
            parse2::<FactoryMacroArgs>(quote! {
                async = true, threadsafe = false, async = false
            }),
            Err(_)
        ));
    }
}
