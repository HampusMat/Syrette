use syn::parse::{Parse, ParseStream};
use syn::{Ident, LitBool, Token};

#[derive(Debug)]
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
