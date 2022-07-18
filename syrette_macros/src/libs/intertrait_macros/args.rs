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
use syn::parse::{Parse, ParseStream, Result};
use syn::{Path, Token, Type};

pub struct Cast
{
    pub ty: Type,
    pub target: Path,
}

impl Parse for Cast
{
    fn parse(input: ParseStream) -> Result<Self>
    {
        let ty: Type = input.parse()?;

        input.parse::<Token![=>]>()?;

        Ok(Cast {
            ty,
            target: input.parse()?,
        })
    }
}
