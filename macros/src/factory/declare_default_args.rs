use syn::parse::Parse;
use syn::punctuated::Punctuated;
use syn::{Token, Type};

use crate::macro_flag::MacroFlag;
use crate::util::iterator_ext::IteratorExt;

pub const FACTORY_MACRO_FLAGS: &[&str] = &["threadsafe", "async"];

pub struct DeclareDefaultFactoryMacroArgs
{
    pub interface: Type,
    pub flags: Punctuated<MacroFlag, Token![,]>,
}

impl Parse for DeclareDefaultFactoryMacroArgs
{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self>
    {
        let interface = input.parse()?;

        if !input.peek(Token![,]) {
            return Ok(Self {
                interface,
                flags: Punctuated::new(),
            });
        }

        input.parse::<Token![,]>()?;

        let flags = Punctuated::<MacroFlag, Token![,]>::parse_terminated(input)?;

        for flag in &flags {
            let name = flag.name().to_string();

            if !FACTORY_MACRO_FLAGS.contains(&name.as_str()) {
                return Err(input.error(format!(
                    "Unknown flag '{name}'. Expected one of [ {} ]",
                    FACTORY_MACRO_FLAGS.join(",")
                )));
            }
        }

        let flag_names = flags
            .iter()
            .map(|flag| flag.name().to_string())
            .collect::<Vec<_>>();

        if let Some((dupe_flag_name, _)) = flag_names.iter().find_duplicate() {
            return Err(input.error(format!("Duplicate flag '{dupe_flag_name}'")));
        }

        Ok(Self { interface, flags })
    }
}

#[cfg(test)]
mod tests
{
    use proc_macro2::Span;
    use quote::{format_ident, quote};
    use syn::token::Dyn;
    use syn::{
        parse2,
        Lit,
        LitBool,
        Path,
        PathArguments,
        PathSegment,
        TraitBound,
        TraitBoundModifier,
        Type,
        TypeParamBound,
        TypeTraitObject,
    };

    use super::*;
    use crate::macro_flag::MacroFlagValue;

    #[test]
    fn can_parse_with_interface_only()
    {
        let input_args = quote! {
            dyn IFoo
        };

        let dec_def_fac_args =
            parse2::<DeclareDefaultFactoryMacroArgs>(input_args).unwrap();

        assert_eq!(
            dec_def_fac_args.interface,
            Type::TraitObject(TypeTraitObject {
                dyn_token: Some(Dyn::default()),
                bounds: Punctuated::from_iter(vec![TypeParamBound::Trait(TraitBound {
                    paren_token: None,
                    modifier: TraitBoundModifier::None,
                    lifetimes: None,
                    path: Path {
                        leading_colon: None,
                        segments: Punctuated::from_iter(vec![PathSegment {
                            ident: format_ident!("IFoo"),
                            arguments: PathArguments::None
                        }])
                    }
                })])
            })
        );

        assert!(dec_def_fac_args.flags.is_empty());
    }

    #[test]
    fn can_parse_with_interface_and_single_flag()
    {
        let input_args = quote! {
            dyn IBar, threadsafe = true
        };

        let dec_def_fac_args =
            parse2::<DeclareDefaultFactoryMacroArgs>(input_args).unwrap();

        assert_eq!(
            dec_def_fac_args.interface,
            Type::TraitObject(TypeTraitObject {
                dyn_token: Some(Dyn::default()),
                bounds: Punctuated::from_iter(vec![TypeParamBound::Trait(TraitBound {
                    paren_token: None,
                    modifier: TraitBoundModifier::None,
                    lifetimes: None,
                    path: Path {
                        leading_colon: None,
                        segments: Punctuated::from_iter(vec![PathSegment {
                            ident: format_ident!("IBar"),
                            arguments: PathArguments::None
                        }])
                    }
                })])
            })
        );

        assert_eq!(
            dec_def_fac_args.flags,
            Punctuated::from_iter(vec![MacroFlag {
                name: format_ident!("threadsafe"),
                value: MacroFlagValue::Literal(Lit::Bool(LitBool::new(
                    true,
                    Span::call_site()
                )))
            }])
        );
    }

    #[test]
    fn can_parse_with_interface_and_multiple_flags()
    {
        let input_args = quote! {
            dyn IBar, threadsafe = true, async = false
        };

        let dec_def_fac_args =
            parse2::<DeclareDefaultFactoryMacroArgs>(input_args).unwrap();

        assert_eq!(
            dec_def_fac_args.interface,
            Type::TraitObject(TypeTraitObject {
                dyn_token: Some(Dyn::default()),
                bounds: Punctuated::from_iter(vec![TypeParamBound::Trait(TraitBound {
                    paren_token: None,
                    modifier: TraitBoundModifier::None,
                    lifetimes: None,
                    path: Path {
                        leading_colon: None,
                        segments: Punctuated::from_iter(vec![PathSegment {
                            ident: format_ident!("IBar"),
                            arguments: PathArguments::None
                        }])
                    }
                })])
            })
        );

        assert_eq!(
            dec_def_fac_args.flags,
            Punctuated::from_iter(vec![
                MacroFlag {
                    name: format_ident!("threadsafe"),
                    value: MacroFlagValue::Literal(Lit::Bool(LitBool::new(
                        true,
                        Span::call_site()
                    )))
                },
                MacroFlag {
                    name: format_ident!("async"),
                    value: MacroFlagValue::Literal(Lit::Bool(LitBool::new(
                        false,
                        Span::call_site()
                    )))
                }
            ])
        );
    }

    #[test]
    fn cannot_parse_with_interface_and_invalid_flag()
    {
        let input_args = quote! {
            dyn IBar, async = true, foo = false
        };

        assert!(parse2::<DeclareDefaultFactoryMacroArgs>(input_args).is_err());
    }

    #[test]
    fn cannot_parse_with_interface_and_duplicate_flag()
    {
        assert!(
            // Formatting is weird without this comment
            parse2::<DeclareDefaultFactoryMacroArgs>(quote! {
                dyn IBar, async = true, threadsafe = false, async = true
            })
            .is_err()
        );

        assert!(
            // Formatting is weird without this comment
            parse2::<DeclareDefaultFactoryMacroArgs>(quote! {
                dyn IBar, async = true, threadsafe = false, async = false
            })
            .is_err()
        );
    }
}
