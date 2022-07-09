/**
 * Originally from Intertrait by CodeChain
 *
 * https://github.com/CodeChain-io/intertrait
 * https://crates.io/crates/intertrait/0.2.2
 *
 * Licensed under either of
 *
 * Apache License, Version 2.0 (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)

 * at your option.
*/
use std::collections::HashSet;

use syn::bracketed;
use syn::parse::{Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::{Error, Ident, Path, Token, Type};

#[derive(Hash, PartialEq, Eq)]
pub enum Flag
{
    Sync,
}

impl Flag
{
    fn from(ident: &Ident) -> Result<Self>
    {
        match ident.to_string().as_str() {
            "sync" => Ok(Flag::Sync),
            unknown => {
                let msg = format!("Unknown flag: {}", unknown);
                Err(Error::new_spanned(ident, msg))
            }
        }
    }
}

pub struct Targets
{
    pub flags: HashSet<Flag>,
    pub paths: Vec<Path>,
}

impl Parse for Targets
{
    fn parse(input: ParseStream) -> Result<Self>
    {
        let mut flags = HashSet::new();
        let mut paths = Vec::new();

        if input.is_empty() {
            return Ok(Targets { flags, paths });
        }

        if input.peek(syn::token::Bracket) {
            let content;
            bracketed!(content in input);
            for ident in Punctuated::<Ident, Token![,]>::parse_terminated(&content)? {
                if !flags.insert(Flag::from(&ident)?) {
                    let msg = format!("Duplicated flag: {}", ident);
                    return Err(Error::new_spanned(ident, msg));
                }
            }
        }

        if input.is_empty() {
            return Ok(Targets { flags, paths });
        }

        paths = Punctuated::<Path, Token![,]>::parse_terminated(input)?
            .into_iter()
            .collect();

        Ok(Targets { flags, paths })
    }
}

pub struct Casts
{
    pub ty: Type,
    pub targets: Targets,
}

impl Parse for Casts
{
    fn parse(input: ParseStream) -> Result<Self>
    {
        let ty: Type = input.parse()?;
        input.parse::<Token![=>]>()?;

        Ok(Casts {
            ty,
            targets: input.parse()?,
        })
    }
}
