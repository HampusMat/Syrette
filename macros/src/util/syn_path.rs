#![allow(clippy::module_name_repetitions)]
use quote::ToTokens;
use syn::punctuated::Pair;

pub fn syn_path_to_string(path: &syn::Path) -> String
{
    path.segments
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
