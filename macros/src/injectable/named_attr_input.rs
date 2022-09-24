use syn::parse::Parse;
use syn::{parenthesized, LitStr};

pub struct NamedAttrInput
{
    pub name: LitStr,
}

impl Parse for NamedAttrInput
{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self>
    {
        let content;

        parenthesized!(content in input);

        Ok(Self {
            name: content.parse()?,
        })
    }
}
