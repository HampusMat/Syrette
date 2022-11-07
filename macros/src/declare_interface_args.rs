use syn::parse::{Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::{Token, TypePath};

use crate::macro_flag::MacroFlag;
use crate::util::iterator_ext::IteratorExt;

pub const DECLARE_INTERFACE_FLAGS: &[&str] = &["async"];

pub struct DeclareInterfaceArgs
{
    pub implementation: TypePath,
    pub interface: TypePath,
    pub flags: Punctuated<MacroFlag, Token![,]>,
}

impl Parse for DeclareInterfaceArgs
{
    fn parse(input: ParseStream) -> Result<Self>
    {
        let implementation: TypePath = input.parse()?;

        input.parse::<Token![->]>()?;

        let interface: TypePath = input.parse()?;

        let flags = if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;

            let flags = Punctuated::<MacroFlag, Token![,]>::parse_terminated(input)?;

            for flag in &flags {
                let flag_str = flag.flag.to_string();

                if !DECLARE_INTERFACE_FLAGS.contains(&flag_str.as_str()) {
                    return Err(input.error(format!(
                        "Unknown flag '{}'. Expected one of [ {} ]",
                        flag_str,
                        DECLARE_INTERFACE_FLAGS.join(",")
                    )));
                }
            }

            let flag_names = flags
                .iter()
                .map(|flag| flag.flag.to_string())
                .collect::<Vec<_>>();

            if let Some(dupe_flag_name) = flag_names.iter().find_duplicate() {
                return Err(input.error(format!("Duplicate flag '{dupe_flag_name}'")));
            }

            flags
        } else {
            Punctuated::new()
        };

        Ok(Self {
            implementation,
            interface,
            flags,
        })
    }
}
