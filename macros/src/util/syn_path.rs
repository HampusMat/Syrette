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
        self.segments
            .pairs()
            .map(Pair::into_tuple)
            .map(|(segment, opt_punct)| {
                let segment_ident = &segment.ident;

                format!(
                    "{}{}",
                    segment_ident,
                    opt_punct.map_or_else(String::new, |punct| punct
                        .to_token_stream()
                        .to_string())
                )
            })
            .collect()
    }
}
