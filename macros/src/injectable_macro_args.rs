use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{braced, Token, TypePath};

use crate::macro_flag::MacroFlag;
use crate::util::iterator_ext::IteratorExt;

pub const INJECTABLE_MACRO_FLAGS: &[&str] = &["no_doc_hidden", "async"];

pub struct InjectableMacroArgs
{
    pub interface: Option<TypePath>,
    pub flags: Punctuated<MacroFlag, Token![,]>,
}

impl Parse for InjectableMacroArgs
{
    fn parse(input: ParseStream) -> syn::Result<Self>
    {
        let interface = input.parse::<TypePath>().ok();

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

        let braced_content;

        braced!(braced_content in input);

        let flags = braced_content.parse_terminated(MacroFlag::parse)?;

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
