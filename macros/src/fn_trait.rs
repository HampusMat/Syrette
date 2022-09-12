use quote::ToTokens;
use syn::parse::Parse;
use syn::punctuated::Punctuated;
use syn::token::Paren;
use syn::{parenthesized, Ident, Token, Type};

/// A function trait. `dyn Fn(u32) -> String`
#[derive(Debug, Clone)]
pub struct FnTrait
{
    pub dyn_token: Token![dyn],
    pub trait_ident: Ident,
    pub paren_token: Paren,
    pub inputs: Punctuated<Type, Token![,]>,
    pub r_arrow_token: Token![->],
    pub output: Type,
}

impl Parse for FnTrait
{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self>
    {
        let dyn_token = input.parse::<Token![dyn]>()?;

        let trait_ident = input.parse::<Ident>()?;

        if trait_ident.to_string().as_str() != "Fn" {
            return Err(syn::Error::new(trait_ident.span(), "Expected 'Fn'"));
        }

        let content;

        let paren_token = parenthesized!(content in input);

        let inputs = content.parse_terminated(Type::parse)?;

        let r_arrow_token = input.parse::<Token![->]>()?;

        let output = input.parse::<Type>()?;

        Ok(Self {
            dyn_token,
            trait_ident,
            paren_token,
            inputs,
            r_arrow_token,
            output,
        })
    }
}

impl ToTokens for FnTrait
{
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream)
    {
        self.dyn_token.to_tokens(tokens);

        self.trait_ident.to_tokens(tokens);

        self.paren_token.surround(tokens, |tokens_inner| {
            self.inputs.to_tokens(tokens_inner);
        });

        self.r_arrow_token.to_tokens(tokens);

        self.output.to_tokens(tokens);
    }
}