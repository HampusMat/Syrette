#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(missing_docs)]

//! Internal macros used by tests.

use std::iter::repeat;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse::Parse;
use syn::{parse_macro_input, LitChar, LitInt, Token};

/// Repeats a character N number of times.
#[proc_macro]
pub fn repeat_char(input: TokenStream) -> TokenStream
{
    let RepeatMacroArgs { character, count } =
        parse_macro_input!(input as RepeatMacroArgs);

    let repeated = repeat(character.value()).take(count).collect::<String>();

    quote! {
        #repeated
    }
    .into()
}

struct RepeatMacroArgs
{
    character: LitChar,
    count: usize,
}

impl Parse for RepeatMacroArgs
{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self>
    {
        let character = input.parse::<LitChar>()?;

        input.parse::<Token![,]>()?;

        let count = input.parse::<LitInt>()?.base10_parse::<usize>()?;

        Ok(Self { character, count })
    }
}
