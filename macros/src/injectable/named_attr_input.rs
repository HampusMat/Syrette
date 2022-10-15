use quote::ToTokens;
use syn::parse::Parse;
use syn::token::Paren;
use syn::{parenthesized, LitStr};

pub struct NamedAttrInput
{
    pub paren: Paren,
    pub name: LitStr,
}

impl Parse for NamedAttrInput
{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self>
    {
        let content;

        let paren = parenthesized!(content in input);

        Ok(Self {
            paren,
            name: content.parse()?,
        })
    }
}

impl ToTokens for NamedAttrInput
{
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream)
    {
        self.paren
            .surround(&mut self.name.to_token_stream(), |stream| {
                stream.to_tokens(tokens);
            });
    }
}
