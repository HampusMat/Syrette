use syn::parse::{Parse, ParseStream};
use syn::{Ident, LitBool, Token};

#[derive(Debug, PartialEq, Eq)]
pub struct MacroFlag
{
    pub flag: Ident,
    pub is_on: LitBool,
}

impl Parse for MacroFlag
{
    fn parse(input: ParseStream) -> syn::Result<Self>
    {
        let input_forked = input.fork();

        let flag: Ident = input_forked.parse()?;

        input.parse::<Ident>()?;

        input.parse::<Token![=]>()?;

        let is_on: LitBool = input.parse()?;

        Ok(Self { flag, is_on })
    }
}

#[cfg(test)]
mod tests
{
    use std::error::Error;

    use proc_macro2::Span;
    use quote::{format_ident, quote};
    use syn::parse2;

    use super::*;

    #[test]
    fn can_parse_macro_flag() -> Result<(), Box<dyn Error>>
    {
        assert_eq!(
            parse2::<MacroFlag>(quote! {
                more = true
            })?,
            MacroFlag {
                flag: format_ident!("more"),
                is_on: LitBool::new(true, Span::call_site())
            }
        );

        assert_eq!(
            parse2::<MacroFlag>(quote! {
                do_something = false
            })?,
            MacroFlag {
                flag: format_ident!("do_something"),
                is_on: LitBool::new(false, Span::call_site())
            }
        );

        assert!(parse2::<MacroFlag>(quote! {
            10 = false
        })
        .is_err());

        Ok(())
    }
}
