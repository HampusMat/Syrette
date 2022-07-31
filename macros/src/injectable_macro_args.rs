use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{braced, Ident, LitBool, Token, TypePath};

use crate::util::iterator_ext::IteratorExt;

pub const INJECTABLE_MACRO_FLAGS: &[&str] = &["no_doc_hidden"];

pub struct InjectableMacroFlag
{
    pub flag: Ident,
    pub is_on: LitBool,
}

impl Parse for InjectableMacroFlag
{
    fn parse(input: ParseStream) -> syn::Result<Self>
    {
        let input_forked = input.fork();

        let flag: Ident = input_forked.parse()?;

        let flag_str = flag.to_string();

        if !INJECTABLE_MACRO_FLAGS.contains(&flag_str.as_str()) {
            return Err(input.error(format!(
                "Unknown flag '{}'. Expected one of [ {} ]",
                flag_str,
                INJECTABLE_MACRO_FLAGS.join(",")
            )));
        }

        input.parse::<Ident>()?;

        input.parse::<Token![=]>()?;

        let is_on: LitBool = input.parse()?;

        Ok(Self { flag, is_on })
    }
}

pub struct InjectableMacroArgs
{
    pub interface: Option<TypePath>,
    pub flags: Punctuated<InjectableMacroFlag, Token![,]>,
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

        let flags = braced_content.parse_terminated(InjectableMacroFlag::parse)?;

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
