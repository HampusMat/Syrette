use syn::parse::{Parse, ParseStream, Result};
use syn::{Path, Token, Type};

pub struct DeclareInterfaceArgs
{
    pub implementation: Type,
    pub interface: Path,
}

impl Parse for DeclareInterfaceArgs
{
    fn parse(input: ParseStream) -> Result<Self>
    {
        let implementation: Type = input.parse()?;

        input.parse::<Token![->]>()?;

        Ok(Self {
            implementation,
            interface: input.parse()?,
        })
    }
}
