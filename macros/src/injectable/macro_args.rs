use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Token, TypePath};

use crate::macro_flag::MacroFlag;
use crate::util::iterator_ext::IteratorExt;

pub const INJECTABLE_MACRO_FLAGS: &[&str] =
    &["no_doc_hidden", "async", "no_declare_concrete_interface"];

pub struct InjectableMacroArgs
{
    pub interface: Option<TypePath>,
    pub flags: Punctuated<MacroFlag, Token![,]>,
}

impl Parse for InjectableMacroArgs
{
    fn parse(input: ParseStream) -> syn::Result<Self>
    {
        let input_fork = input.fork();

        let mut interface = None;

        if input_fork.parse::<MacroFlag>().is_err() {
            // Input doesn't begin with flags

            interface = input.parse::<TypePath>().ok();

            if interface.is_some() {
                let comma_input_lookahead = input.lookahead1();

                if !comma_input_lookahead.peek(Token![,]) {
                    return Ok(Self {
                        interface,
                        flags: Punctuated::new(),
                    });
                }

                input.parse::<Token![,]>()?;
            }

            if input.is_empty() {
                return Ok(Self {
                    interface,
                    flags: Punctuated::new(),
                });
            }
        }

        let flags = Punctuated::<MacroFlag, Token![,]>::parse_terminated(input)?;

        for flag in &flags {
            let flag_str = flag.flag.to_string();

            if !INJECTABLE_MACRO_FLAGS.contains(&flag_str.as_str()) {
                return Err(input.error(format!(
                    "Unknown flag '{}'. Expected one of [ {} ]",
                    flag_str,
                    INJECTABLE_MACRO_FLAGS.join(",")
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

        Ok(Self { interface, flags })
    }
}
