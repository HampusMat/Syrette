use syn::parse::{Parse, ParseStream};
use syn::TypePath;

pub struct InjectableMacroArgs
{
    pub interface: TypePath,
}

impl Parse for InjectableMacroArgs
{
    fn parse(input: ParseStream) -> syn::Result<Self>
    {
        Ok(Self {
            interface: input.parse()?,
        })
    }
}
