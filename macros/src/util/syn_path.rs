use std::fmt::Write;

use quote::ToTokens;
use syn::punctuated::Pair;

pub trait SynPathExt
{
    /// Converts the [`syn::Path`] to a [`String`].
    fn to_string(&self) -> String;
}

impl SynPathExt for syn::Path
{
    fn to_string(&self) -> String
    {
        self.segments.pairs().map(Pair::into_tuple).fold(
            String::new(),
            |mut acc, (segment, opt_punct)| {
                let segment_ident = &segment.ident;

                write!(
                    acc,
                    "{segment_ident}{}",
                    opt_punct
                        .map(|punct| punct.to_token_stream().to_string())
                        .unwrap_or_default()
                )
                .ok();

                acc
            },
        )
    }
}
