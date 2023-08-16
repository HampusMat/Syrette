use syn::parse::Parse;
use syn::punctuated::Punctuated;
use syn::Token;

use crate::macro_flag::MacroFlag;
use crate::util::iterator_ext::IteratorExt;

pub const FACTORY_MACRO_FLAGS: &[&str] = &["threadsafe"];

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
            let name = flag.name().to_string();

            if !FACTORY_MACRO_FLAGS.contains(&name.as_str()) {
                return Err(input.error(format!(
                    "Unknown flag '{}'. Expected one of [ {} ]",
                    name,
                    FACTORY_MACRO_FLAGS.join(",")
                )));
            }
        }

        let flag_names = flags
            .iter()
            .map(|flag| flag.name().to_string())
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
    use syn::{parse2, Lit, LitBool};

    use super::*;
    use crate::macro_flag::MacroFlagValue;

    #[test]
    fn can_parse_with_single_flag() -> Result<(), Box<dyn Error>>
    {
        let input_args = quote! {
            threadsafe = true
        };

        let factory_macro_args = parse2::<FactoryMacroArgs>(input_args)?;

        assert_eq!(
            factory_macro_args.flags,
            Punctuated::from_iter(vec![MacroFlag {
                name: format_ident!("threadsafe"),
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
            threadsafe = false, foo = true
        };

        assert!(parse2::<FactoryMacroArgs>(input_args).is_err());
    }

    #[test]
    fn cannot_parse_with_duplicate_flag()
    {
        assert!(
            // Formatting is weird without this comment
            parse2::<FactoryMacroArgs>(quote! {
                threadsafe = true, threadsafe = true
            })
            .is_err()
        );

        assert!(
            // Formatting is weird without this comment
            parse2::<FactoryMacroArgs>(quote! {
                threadsafe = true, threadsafe = false
            })
            .is_err()
        );
    }
}
