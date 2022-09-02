use syn::parse::Parse;
use syn::punctuated::Punctuated;
use syn::Token;

use crate::macro_flag::MacroFlag;
use crate::util::iterator_ext::IteratorExt;

pub const FACTORY_MACRO_FLAGS: &[&str] = &["threadsafe"];

pub struct FactoryMacroArgs
{
    pub flags: Punctuated<MacroFlag, Token![,]>,
}

impl Parse for FactoryMacroArgs
{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self>
    {
        let flags = Punctuated::<MacroFlag, Token![,]>::parse_terminated(input)?;

        for flag in &flags {
            let flag_str = flag.flag.to_string();

            if !FACTORY_MACRO_FLAGS.contains(&flag_str.as_str()) {
                return Err(input.error(format!(
                    "Unknown flag '{}'. Expected one of [ {} ]",
                    flag_str,
                    FACTORY_MACRO_FLAGS.join(",")
                )));
            }
        }

        let flag_names = flags
            .iter()
            .map(|flag| flag.flag.to_string())
            .collect::<Vec<_>>();

        if let Some(dupe_flag_name) = flag_names.iter().find_duplicate() {
            return Err(input.error(format!("Duplicate flag '{}'", dupe_flag_name)));
        }

        Ok(Self { flags })
    }
}
